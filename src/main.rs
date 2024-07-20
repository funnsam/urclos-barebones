#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

mod gdt;
mod serial;

mod ata;
mod fs;

mod keyboard;
mod interrupts;

mod console;
mod vga_buffer;

#[cfg(feature = "ramfs")]
mod ramfs;
mod bindings;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    fs::set_disk();

    unsafe { bindings::urcl_main(); }

    fs::flush_changes();
    reboot();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    println!("{}", info);
    hlt_loop();
}

fn init() {
    vga_buffer::init();
    serial::init();
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn reboot() -> ! {
    let mut t = 2_u8;
    let mut p = x86_64::instructions::port::Port::new(0x64);
    while t & 2 != 0 {
        t = unsafe { p.read() };
    }
    unsafe { p.write(0xfe); }
    hlt_loop();
}
