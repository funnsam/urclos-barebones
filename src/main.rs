#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::{panic::PanicInfo, sync::atomic::Ordering};

mod gdt;
mod serial;

mod ata;

mod keyboard;
mod interrupts;

mod console;
mod vga_buffer;

mod fs;
mod bindings;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();

    if ata::check_exists() {
        println!("\x1b[33mDrive detected! Use it instead of ramfs? [Y/*]\x1b[0m");

        if comfirm() {
            bindings::USE_DISK.store(true, Ordering::Relaxed);
            println!("\x1b[32mOK\x1b[0m");

            ata::initialize_read(false, 0, 1);
            let d = ata::get_next_chunk();
            for d in d.chunks_exact(16) {
                for d in d.iter() {
                    print!("{d:04x} ");
                }
                println!();
            }
        }
    }

    unsafe { bindings::urcl_main(); }
    println!("\x1b[1mURCL-OS halted\x1b[0m");

    if bindings::USE_DISK.load(Ordering::Relaxed) {
        println!("\x1b[33mSave fs to disk? \x1b[31;1mAny data which was on the disk may get overwritten! [Y/*]\x1b[0m");

        if comfirm() {
            println!("\x1b[32mSaving\x1b[0m");
            let _ = ata::flush_cache();
        }
    }

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
