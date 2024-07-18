#![allow(const_item_mutation)]

use x86_64::instructions::port::*;

const ATA_PRIM_BUS: u16 = 0x1f0;
const ATA_PRIM_BUS0: Port<u16> = Port::new(ATA_PRIM_BUS + 0);
const ATA_PRIM_BUS1: Port<u8> = Port::new(ATA_PRIM_BUS + 1);
const ATA_PRIM_BUS2: Port<u8> = Port::new(ATA_PRIM_BUS + 2);
const ATA_PRIM_BUS3: Port<u8> = Port::new(ATA_PRIM_BUS + 3);
const ATA_PRIM_BUS4: Port<u8> = Port::new(ATA_PRIM_BUS + 4);
const ATA_PRIM_BUS5: Port<u8> = Port::new(ATA_PRIM_BUS + 5);
const ATA_PRIM_BUS6: Port<u8> = Port::new(ATA_PRIM_BUS + 6);
const ATA_PRIM_BUS7: Port<u8> = Port::new(ATA_PRIM_BUS + 7);
const ATA_PRIM_CTRL: Port<u8> = Port::new(0x3f6);

fn _check_exists() -> Result<(), ()> {
    check_exists().then_some(()).ok_or(())
}

pub fn check_exists() -> bool {
    (unsafe { ATA_PRIM_BUS7.read() }) != 0xff
}

pub fn flush_cache() -> Result<(), ()> {
    _check_exists()?;
    unsafe { ATA_PRIM_CTRL.write(0xe7); }
    wait_busy_clear();
    Ok(())
}

fn wait_busy_clear() {
    loop {
        let p = unsafe { ATA_PRIM_BUS7.read() };

        if p & 0x88 == 8 || p & 0xa0 == 0x20 || p & 0x81 == 1 {
            break;
        }
    }
}

pub fn has_err() -> bool {
    (unsafe { ATA_PRIM_BUS7.read() }) & 0x21 != 0
}

fn _has_err() -> Result<(), ()> {
    has_err().then_some(()).ok_or(())
}

pub fn initialize_read(main: bool, lba: u32, sectors: u8) {
    unsafe {
        ATA_PRIM_BUS6.write(0xe0 | (((!main) as u8) << 4) | ((lba >> 24) as u8 & 0xf));
        ATA_PRIM_BUS1.write(0x00);
        ATA_PRIM_BUS2.write(sectors);
        ATA_PRIM_BUS3.write(lba as u8);
        ATA_PRIM_BUS4.write((lba >> 8) as u8);
        ATA_PRIM_BUS5.write((lba >> 16) as u8);
        ATA_PRIM_BUS7.write(0x20);
    }
}

pub fn get_next_chunk() -> [u16; 256] {
    unsafe {
        wait_busy_clear();

        let mut buf = [0; 256];
        for i in buf.iter_mut() {
            *i = ATA_PRIM_BUS0.read();
        }

        buf
    }
}
