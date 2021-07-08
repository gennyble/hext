# hxt
`hxt` is the command-line tool that utilizes the `hext` crate. It acts like
`cat` meaning if you pass no file name, it reads input from stdin. If you pass
multiple file names, it will process them in order.

You may specify an output file with the `-o` option. If no output file is
specified, hxt will output to stdout.

```
Usage: hxt [options] FILES

Options:
    -o, --output FILE   output to a file
    -h, --help          print this message and exit
```
