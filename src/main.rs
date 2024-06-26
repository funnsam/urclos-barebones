#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

mod gdt;
mod serial;

mod keyboard;
mod interrupts;

mod console;
mod vga_buffer;

mod fs;
mod bindings;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    unsafe { bindings::urcl_main(); }
    println!("URCL-OS halted");
    hlt_loop();
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
