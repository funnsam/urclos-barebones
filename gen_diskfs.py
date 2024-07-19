#!/bin/python3

import sys
import os

file = sys.argv[1]
fs = sys.argv[2]
out = bytearray()

with open(file, "rb") as f:
    while (b := f.read(2)):
        out.append(b[1])
        out.append(b[0])

with open(fs, "wb") as f:
    f.write(b"\0\0" * 256)
    f.write(out)

print("\x1b[1;34mDo `n`, `t 2c` and `w`\x1b[0m")
os.system(f"fdisk -u {fs}")
