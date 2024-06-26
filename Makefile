build: liburcl.a src/fs.rs
	cargo r -r

urcl.o:
	urcl-llvm urcl-os/urclos3.urcl --triple x86_64-none-elf

liburcl.a: urcl.o
	i686-elf-ar -rcs liburcl.a urcl.o

src/fs.rs:
	python3 gen_urclos_fs.py urcl-os/fs.bin

clean:
	rm *.o *.a

.PHONY: build clean
