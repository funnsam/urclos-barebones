#!/bin/python3

import sys
import os

file = sys.argv[1]
out = ""

with open(file, "rb") as f:
    f.seek(0, os.SEEK_END)
    out = f"""\
// generated by get_urclos_fs.py for {file}
#[rustfmt::skip]
pub static mut FS: [u16; {f.tell() >> 1}] = ["""
    f.seek(0)

    while (b := f.read(2)):
        out += "0x%02x%02x," % (b[0], b[1])

out += "];"

with open("src/ramfs.rs", "w") as f:
    f.write(out)