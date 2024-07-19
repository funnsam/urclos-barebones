use core::sync::atomic::*;

use crate::*;

static USE_DISK: AtomicBool = AtomicBool::new(false);
static USE_SLAVE: AtomicBool = AtomicBool::new(false);
static START_LBA: AtomicU32 = AtomicU32::new(0);
static SECTORS: AtomicU32 = AtomicU32::new(0);

pub fn set_disk() {
    let mut part = [None; 8];
    print_disk(false, &mut part, 0);
    print_disk(true, &mut part, 4);

    if part.iter().any(|p| p.is_some()) {
        loop {
            println!("\x1b[33mSelect a partition as filesystem [0-7/*]\x1b[0m");
            let s = keyboard::poll_key() - b'0' as u32;

            if let Some(Some(p)) = part.get(s as usize) {
                USE_DISK.store(true, Ordering::Relaxed);
                USE_SLAVE.store(s >= 4, Ordering::Relaxed);
                START_LBA.store(p.0, Ordering::Relaxed);
                SECTORS.store(p.1, Ordering::Relaxed);

                println!("\x1b[32;1mUsing partition #{s}\x1b[0m");
                return;
            } else {
                #[cfg(not(feature = "ramfs"))] {
                    println!("\x1b[31;1mInvalid partition\x1b[0m");
                }
            }
        }
    } else {
        #[cfg(not(feature = "ramfs"))] {
            println!("\x1b[31;1mNo partition usable\x1b[0m");
            crate::hlt_loop();
        }
    }
}

fn print_disk(slave: bool, part: &mut [Option<(u32, u32)>], start_idx: usize) {
    ata::initialize_read(slave, 0, 1);

    if let Ok(mbr) = ata::get_next_chunk() {
        let id = (mbr[0xdc] as u32) | ((mbr[0xdd] as u32) << 16);
        println!("\x1b[32;1mDisk {}:\x1b[0m (id {id:08x})", slave as u8);

        for (pi, _p) in mbr[0xdf..].chunks_exact(8).take(4).enumerate() {
            let mut p = [0; 16];
            for (p, b) in p.iter_mut().zip(_p.into_iter().flat_map(|w| w.to_le_bytes())) {
                *p = b;
            }

            if p[0x4] != 0x2c { continue; }

            let lba = u32::from_le_bytes(p[0x8..0xc].try_into().unwrap());
            let sec = u32::from_le_bytes(p[0xc..0x10].try_into().unwrap());
            println!("  \x1b[1m[{}]:\x1b[0m Partition {pi}: LBA {lba}, {sec} sectors", start_idx + pi);

            part[start_idx + pi] = Some((lba, sec));
        }
    }
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
