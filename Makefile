CC_FLAGS=-Wpedantic -Wall -Wextra -std=gnu11 -O0 -ffreestanding
CC=i686-elf-gcc
LD_FLAGS=-ffreestanding -O0 -nostdlib -lgcc
LD=i686-elf-gcc
AS_FLAGS=
AS=i686-elf-as

test: urclos.iso
	qemu-system-i386 -cdrom urclos.iso

urclos_fs.h:
	python3 gen_urclos_fs.py urcl-os/fs.bin

boot.o:
	$(AS) boot.s -o boot.o $(AS_FLAGS)

urcl.o:
	# urcl-llvm urcl-os/urclos3.urcl --triple i686-pc-none-elf
	urcl-llvm urcl.urcl --triple i686-pc-none-elf

gdt.o:
	$(CC) gdt.c -c -o gdt.o $(CC_FLAGS)

kernel.o: urclos_fs.h
	$(CC) kernel.c -c -o kernel.o $(CC_FLAGS)

urclos.bin: boot.o kernel.o urcl.o gdt.o
	$(LD) -T linker.ld -o urclos.bin boot.o kernel.o urcl.o gdt.o $(LD_FLAGS)

urclos.iso: urclos.bin
	- mkdir -p isodir/boot/grub
	cp urclos.bin isodir/boot/urclos.bin
	cp grub.cfg isodir/boot/grub/grub.cfg
	grub-mkrescue -o urclos.iso isodir
