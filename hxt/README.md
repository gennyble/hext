# hxt
`hxt` is the command-line tool that utilizes the `hext` crate. It acts like
`cat` meaning if you pass no file name, it reads input from stdin. If you pass
multiple file names, it will process them in order.

You may specify an output file with the `-o` option. If no output file is
specified, hxt will output to stdout.

By default if you do not complete an octet when writing binary (using the `.`),
hxt will terminate and return an error. If you pass the `-p` flag (short for
`--pad-bits`), then hxt will fill the rest of the octet assuming the most
significant bit is first. What does that mean? If you have an unaligned set
of bits, say `.1`, it will get padded as if it was wrote as `.10000000`. So,
if your file ends with `.1` or if you start writing hex again without
completing a bit octet (like this `.1 40`), then you'll get `0x7F` and
`0x7F 0x40`, respectively.

```
Usage: hxt [options] FILES

Options:
    -p, --pad-bits      pad bits until they reach an octet
    -o, --output FILE   output to a file
    -h, --help          print this message and exit
```
