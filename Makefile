URCL=urcl-os/urclos3.urcl
RAMFS=urcl-os/fs.bin

build: liburcl.a src/ramfs.rs
	cargo r -r

urcl.o:
	urcl-llvm $(URCL) --triple x86_64-unknown-none --features ",-mmx,-sse,-bmi2,+soft-float" -O3 --max-ram 65536 --use-global

liburcl.a: urcl.o
	ar -rcs liburcl.a urcl.o

src/ramfs.rs: gen_ramfs.py $(RAMFS)
	python3 gen_ramfs.py $(RAMFS)

clean:
	- rm *.o *.a

.PHONY: build clean
