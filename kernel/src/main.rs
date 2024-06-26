#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader_api::{entry_point, BootInfo};

mod console;
mod psf;
mod gdt;
mod interrupts;
mod serial;

entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

fn init(bi: &'static mut BootInfo) {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    unsafe { console::init(bi); }
}

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
