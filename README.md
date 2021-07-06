# UTF-8 extractor
Extract null-terminated UTF-8 strings from any file.

Intended as a replacement for [GNU
`strings`](https://linux.die.net/man/1/strings), providing these missing
features I required:

  * UTF-8 support
  * useful output (offset, length)
  * limit to null-terminated strings
