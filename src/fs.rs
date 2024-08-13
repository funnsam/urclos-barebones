use core::sync::atomic::*;

use crate::*;

static USE_DISK: AtomicBool = AtomicBool::new(false);
static USE_SLAVE: AtomicBool = AtomicBool::new(false);
static START_LBA: AtomicU32 = AtomicU32::new(0);
static SECTORS: AtomicU32 = AtomicU32::new(0);

#[cfg(feature = "atafs")]
pub fn set_disk() {
    let mut parts = [(0, 0); 10];
    let mut partc = 0;

    println!("Scanning disks");
    print_disk(false, &mut parts, &mut partc);
    print_disk(true, &mut parts, &mut partc);

    match partc {
        2.. => loop {
            print!("\n\x1b[36;1mSelect a partition as filesystem [0-{}]\x1b[0m ", partc - 1);

            let mut s = 0;
            loop {
                let k = keyboard::poll_key() as u8;
                match k {
                    b'0'..=b'9' => {
                        s *= 10;
                        s += (k - b'0') as usize;
                        print!("{}", k as char);
                    },
                    b'\n' => break,
                    0x7f => {
                        s /= 10;
                        print!("\x1b[1D \x1b[1D");
                    },
                    _ => {},
                }
            }

            println!();

            if let Some(p) = parts[..partc].get(s) {
                use_disk(s, *p);
                return;
            } else {
                #[cfg(not(feature = "ramfs"))] {
                    println!("\x1b[31;1mInvalid partition #{s}\x1b[0m");
                }
            }
        },
        1 => {
            use_disk(0, parts[0]);
        },
        0 => {
            #[cfg(not(feature = "ramfs"))] {
                println!("\n\x1b[31;1mNo partition usable\x1b[0m");
                crate::hlt_loop();
            }
        },
    }
}

#[cfg(not(feature = "atafs"))]
pub fn set_disk() {}

fn use_disk(s: usize, p: (u32, u32)) {
    USE_DISK.store(true, Ordering::Relaxed);
    USE_SLAVE.store(s >= 4, Ordering::Relaxed);
    START_LBA.store(p.0, Ordering::Relaxed);
    SECTORS.store(p.1, Ordering::Relaxed);

    println!("\x1b[32;1mUsing partition #{s}\x1b[0m\n");
}

fn print_disk(slave: bool, parts: &mut [(u32, u32)], partc: &mut usize) {
    if let Ok(Some(id)) = ata::identify(slave) {
        let sec = (id[60] as u32) | ((id[61] as u32) << 16);
        println!("\x1b[32;1mDisk {}:\x1b[0m {}KiB", slave as u8, sec >> 1);

        print_parts(slave, 0, 0, 1, parts, partc)
    }
}

fn print_parts(slave: bool, lba: u32, lbar: u32, poff: usize, parts: &mut [(u32, u32)], partc: &mut usize) {
    ata::initialize_read(slave, lba, 1);

    if let Ok(mbr) = ata::get_next_chunk() {
        if mbr[0xde] == 0x5a5a {
            println!("  \x1b[33;1mDisk marked as read only\x1b[0m");
            return;
        }

        for (pi, _p) in mbr[0xdf..].chunks_exact(8).take(4).enumerate() {
            let mut p = [0; 16];
            for (p, b) in p.iter_mut().zip(_p.into_iter().flat_map(|w| w.to_le_bytes())) {
                *p = b;
            }

            match p[0x4] {
                0x2c => register(lba, pi + poff, &p, parts, partc),
                0x05 | 0x0f => {
                    let lba = u32::from_le_bytes(p[0x8..0xc].try_into().unwrap()) + lbar;
                    print_parts(slave, lba, if lbar == 0 { lba } else { lbar }, poff.max(4) + 1, parts, partc);
                },
                _ => continue,
            }
        }
    } else {
        println!("  \x1b[31;1mFailed to read MBR\x1b[0m");
    }
}

fn register(lba: u32, pi: usize, p: &[u8], parts: &mut [(u32, u32)], partc: &mut usize) {
    let lba = u32::from_le_bytes(p[0x8..0xc].try_into().unwrap()) + lba;
    let sec = u32::from_le_bytes(p[0xc..0x10].try_into().unwrap());
    let kib = sec >> 1;

    println!("  \x1b[1m[{partc}]:\x1b[0m Partition {pi}: LBA {lba}, {kib}KiB");

    parts[*partc] = (lba, sec);
    *partc += 1;
}

static mut DISK_CACHE: [u16; 256] = [0; 256];
static mut DISK_CACHE_LBA: u32 = u32::MAX;
static mut DISK_CHANGED: bool = false;

fn swap_to(lba: u32) {
    unsafe {
        if DISK_CACHE_LBA != lba {
            if DISK_CHANGED {
                flush_changes();
            }

            ata::initialize_read(USE_SLAVE.load(Ordering::Relaxed), lba, 1);
            let chunk = ata::get_next_chunk().unwrap();
            DISK_CACHE = chunk;
            DISK_CACHE_LBA = lba;
        }
    }
}

pub fn read(addr: usize) -> u16 {
    #[cfg(feature = "ramfs")]
    if !USE_DISK.load(Ordering::Relaxed) { return unsafe { ramfs::FS[addr] }; }

    unsafe {
        if (addr >> 8) as u32 >= SECTORS.load(Ordering::Relaxed) {
            panic!("fs read oob {addr:08x}");
        }

        let lba = (addr >> 8) as u32 + START_LBA.load(Ordering::Relaxed);
        swap_to(lba);
        return DISK_CACHE[addr & 0xff];
    }
}

pub fn write(addr: usize, data: u16) {
    #[cfg(feature = "ramfs")]
    if !USE_DISK.load(Ordering::Relaxed) { return unsafe { ramfs::FS[addr] = data }; }

    unsafe {
        let lba = (addr >> 8) as u32 + START_LBA.load(Ordering::Relaxed);
        swap_to(lba);
        DISK_CACHE[addr & 0xff] = data;
        DISK_CHANGED = true;
    }
}

pub fn flush_changes() {
    unsafe {
        if USE_DISK.load(Ordering::Relaxed) && DISK_CHANGED {
            ata::write_disk(
                USE_SLAVE.load(Ordering::Relaxed),
                DISK_CACHE_LBA,
                &DISK_CACHE,
            ).unwrap();
            DISK_CHANGED = false;
        }
    }
}
