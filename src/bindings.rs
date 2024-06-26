use core::sync::atomic::*;
use crate::*;

#[link(name = "urcl")]
extern "C" {
    pub fn urcl_main();
}

static ADDRESS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub extern "C" fn urcl_in(port: usize) -> usize {
    match port {
        1 => loop {
            if let Some(k) = keyboard::read_key() {
                return k as _;
            }
        },
        32 => ADDRESS.load(Ordering::Relaxed),
        33 => unsafe { fs::FS[ADDRESS.load(Ordering::Relaxed)] as usize },
        _ => {
            println!("todo port {port}");
            loop {}
        },
    }
}

#[no_mangle]
pub extern "C" fn urcl_out(port: usize, data: usize) {
    match port {
        1 => print!("{}", data as u8 as char),
        32 => { ADDRESS.store(data, Ordering::Relaxed); },
        33 => unsafe { fs::FS[ADDRESS.load(Ordering::Relaxed)] = data as u16; },
        _ => println!("todo port {port}"),
    }
}
