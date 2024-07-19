#!/bin/python3

import sys

inp = sys.argv[1]
out = sys.argv[2]

with open(out, "ab") as o:
    with open(inp, "rb") as i:
        while (b := i.read(2)):
            o.write(bytes([b[1], b[0]]))
