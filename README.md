# hext
Do you find yourself exploring file formats, such as GIF, and finding yourself
annoyed with hex editors? Do you find yourself wanting to craft 802.11 packets,
but you know you'll never be able to keep all the fields straight in your head?
Write those packets in hext! hext (`*.hxt`) is a file format for writing small
(or large, if you dare) binary files with ease. Just gaze at the beauty below.

```
515745525459 # `QWERTY`, the most uesd keyboard layout
10 13        # Linefeed (LF) and Carriage Return (CF)
.01000001    # The letter A, in Binary, just to demonstrate how binary can be used
.0100 .0010  # B. Binary doesn't need to be grouped in octets
10
```

The above file is the same as
```
QWERTY
A
```
