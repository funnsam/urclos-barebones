#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{panic::PanicInfo, sync::atomic::Ordering};

mod gdt;
mod serial;

mod ata;
mod fs;

mod keyboard;
mod interrupts;

mod console;
mod vga_buffer;

mod ramfs;
mod bindings;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    fs::set_disk();

    unsafe { bindings::urcl_main(); }
    println!("\x1b[1mURCL-OS halted\x1b[0m");

    hlt_loop();
}

fn comfirm() -> bool {
    loop {
        if let Some(k) = keyboard::read_key() {
            return k == 'Y' as u32;
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

fn init() {
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
