URCL=urcl-os/urclos3.urcl
RAMFS=urcl-os/fs.bin

build: liburcl.a src/ramfs.rs
	cargo r -r

urcl.o:
	urcl-llvm $(URCL) --triple x86_64-unknown-none --features ",-mmx,-sse,-bmi2,+soft-float" -O3 --max-ram 65536 --use-global

liburcl.a: urcl.o
	ar -rcs $@ $<

src/ramfs.rs: gen_ramfs.py $(RAMFS)
	python3 gen_ramfs.py $(RAMFS)

fs.img: to_le.py
	- rm fs.img
	dd if=/dev/zero of=$@ bs=512 count=1
	python3 to_le.py urcl-os/fs.bin $@
	echo -e "o\nn\n\n\n\n\nt\n2c\nw\n" | fdisk $@

clean:
	- rm *.o *.a fs.img src/ramfs.rs

.PHONY: build clean
