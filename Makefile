URCLOS_PATH=urcl-os/urclos3.urcl
FS_PATH=urcl-os/fs.bin

run-ide: urclos.img
	qemu-system-x86_64 -serial stdio \
		-drive file=urclos.img,if=ide,format=raw

urcl.o: $(URCLOS_PATH)
	urcl-llvm $(URCLOS_PATH) --triple x86_64-unknown-none --features ",-mmx,-sse,-bmi2,+soft-float" -O3 --max-ram 65536 --use-global

liburcl.a: urcl.o
	ar -rcs $@ $<

src/ramfs.rs: gen_ramfs.py $(FS_PATH)
	python3 gen_ramfs.py $(FS_PATH)

fs.img: to_le.py $(FS_PATH)
	dd if=/dev/zero of=$@ bs=512 count=1
	python3 to_le.py $(FS_PATH) $@
	echo -e "o\nn\n\n\n\n\nt\n2c\nw\n" | fdisk $@

urclos.img: liburcl.a src/* Cargo.toml Cargo.lock build.rs x86_64-blog_os.json $(FS_PATH)
	cargo bootimage -r
	
	cp target/x86_64-blog_os/release/bootimage-urclos.bin $@
	echo -e "n\n\n4\n\n\nw\n" | fdisk $@
	
	python3 to_le.py $(FS_PATH) $@
	echo -e "n\n\n\n\n\nt\n1\n2c\nd\n4\nw\n" | fdisk $@

clean:
	- rm *.o *.a *.img src/ramfs.rs

.PHONY: run-ide clean
