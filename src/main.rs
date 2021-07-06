/*
TODO:
  * actually it's a little bit more complex if you want to be perfect, should
    just check value lol, much easier
  * https://www.fileformat.info/info/unicode/utf8.htm
  * https://www.cl.cam.ac.uk/~mgk25/ucs/examples/UTF-8-test.txt
  * https://en.wikipedia.org/wiki/UTF-8
  * https://stackoverflow.com/questions/1301402/example-invalid-utf8-string

We don't aim to be perfect and check for invalid overlong encodings. But we do
due diligence.
*/

use std::io::BufReader;
use std::slice;
use std::io::Read;
use std::io::Seek;
use std::fs::File;
use std::io::SeekFrom;
use std::convert::TryInto;

struct FileCursor {
    reader: BufReader<File>,
    str_start: u64,
    str_bytelen: u64,
    str_char_num: u64,
}

fn main() -> std::io::Result<()> {
    let f = File::open("res/valid-utf8-one-of-each-length-char.bin")?;
    let reader = BufReader::new(f);
    let mut cursor = FileCursor {
        reader: reader,
        str_start: 0,
        str_bytelen: 0,
        str_char_num: 0,
    };

    loop {
        let mut byte: u8 = 0x00;
        let bytes_read = cursor.reader.read(slice::from_mut(&mut byte))?;
        if bytes_read == 0 {
            break;
        } else {
            cursor = process_byte(cursor, byte)?;
        }
    }

    Ok(())
}

fn process_byte(mut cursor: FileCursor, byte: u8) -> std::io::Result<FileCursor> {
    if byte == 0x00 && cursor.str_bytelen != 0 {
        cursor.reader.seek(SeekFrom::Start(cursor.str_start));
        let mut bytestr = vec![0; cursor.str_bytelen.try_into().unwrap()];
        cursor.reader.read_exact(&mut bytestr[..]);
        cursor.reader.seek(SeekFrom::Current(1));
        println!("0x{:08x}  0x{:04x}    {}", cursor.str_start, cursor.str_bytelen, cursor.str_char_num);
        println!("{:?}", bytestr);
        cursor = cursor_reset_string(cursor);
    } else if is_ascii(byte) {
        cursor.str_bytelen += 1;
        cursor.str_char_num += 1;
    } else {
        match try_get_utf8_multibyte_len(byte) {
            None => cursor = cursor_reset_string(cursor),
            Some(cont_bytes) => {
                println!("multibyte: {}", cont_bytes);
                // do the bookkeeping early, overwrite if error
                // TODO instead, return an Option wrapped in an io::Result
                cursor.str_bytelen += u64::from(cont_bytes+1);
                cursor.str_char_num += 1;
                cursor = process_multibyte_char(cursor, cont_bytes)?;
            },
        };
    };
    Ok(cursor)
}

fn cursor_reset_string(mut cursor: FileCursor) -> FileCursor {
    cursor.str_start += cursor.str_bytelen+1;
    cursor.str_bytelen = 0;
    cursor.str_char_num = 0;
    cursor
}

// cont_bytes must be 0-3 inclusive
fn process_multibyte_char(mut cursor: FileCursor, cont_bytes: u8) -> std::io::Result<FileCursor> {
    for _ in 0..cont_bytes {
        let mut byte: u8 = 0x00;
        let bytes_read = cursor.reader.read(slice::from_mut(&mut byte))?;
        if bytes_read == 0 {
            println!("EOF during multibyte char");
            break;
        } else {
            if !is_continuation_byte(byte) {
                cursor = cursor_reset_string(cursor);
            }
        }
    }
    Ok(cursor)
}

fn is_ascii(byte: u8) -> bool {
    !bit_at(byte, 7)
}

// is allowed to assume 0x1XXXXXXX
// we try hard enough, ignoring the restricted higher values for 4-bytes
fn try_get_utf8_multibyte_len(byte: u8) -> Option<u8> {
    if byte >= 0xC2 && byte <= 0xDF {
        Some(1)
    } else if byte >= 0xE0 && byte <= 0xEF {
        Some(2)
    } else if byte >= 0xF0 && byte <= 0xF4 {
        Some(3)
    } else {
        None
    }
}

fn is_continuation_byte(byte: u8) -> bool {
    let masked_byte = byte & 0b1000_0000;
    bit_at(masked_byte, 7) && !bit_at(masked_byte, 6)
}

// i must be 0-7 (LSB-MSB) inclusive
fn bit_at(byte: u8, i: u8) -> bool {
    byte & (0b0000_0001 << i) != 0
}
