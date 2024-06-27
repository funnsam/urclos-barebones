URCL=urcl-os/urclos3.urcl

build: liburcl.a src/fs.rs
	cargo r -r

urcl.o:
	urcl-llvm $(URCL) --triple x86_64-unknown-none --features ",-mmx,-sse,-bmi2,+soft-float" -O3 --max-ram 65536 --use-global

liburcl.a: urcl.o
	ar -rcs liburcl.a urcl.o

src/fs.rs:
	python3 gen_urclos_fs.py urcl-os/fs.bin

clean:
	- rm *.o *.a

.PHONY: build clean
