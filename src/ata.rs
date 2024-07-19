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

// pub fn check_exists() -> bool {
//     (unsafe { ATA_PRIM_BUS7.read() }) != 0xff
// }
// 
// fn _check_exists() -> Result<(), ()> {
//     check_exists().then_some(()).ok_or(())
// }
// 
// pub fn flush_cache() -> Result<(), ()> {
//     _check_exists()?;
//     unsafe { ATA_PRIM_CTRL.write(0xe7); }
//     wait_busy_clear();
//     Ok(())
// }

fn wait_busy_clear() {
    while unsafe { ATA_PRIM_BUS7.read() } & 0x80 != 0 {}
}

fn wait_ok_or_err() {
    while unsafe { ATA_PRIM_BUS7.read() } & 9 == 0 {}
}

pub fn has_err() -> bool {
    (unsafe { ATA_PRIM_BUS7.read() }) & 1 != 0
}

fn _has_err() -> Result<(), ()> {
    (!has_err()).then_some(()).ok_or(())
}

pub fn initialize_read(slave: bool, lba: u32, sectors: u8) {
    unsafe {
        ATA_PRIM_BUS6.write(0xe0 | ((slave as u8) << 4) | ((lba >> 24) as u8 & 0xf));
        ATA_PRIM_BUS1.write(0x00);
        ATA_PRIM_BUS2.write(sectors);
        ATA_PRIM_BUS3.write(lba as u8);
        ATA_PRIM_BUS4.write((lba >> 8) as u8);
        ATA_PRIM_BUS5.write((lba >> 16) as u8);
        ATA_PRIM_BUS7.write(0x20);
    }
}

pub fn get_next_chunk() -> Result<[u16; 256], ()> {
    unsafe {
        wait_busy_clear();
        _has_err()?;
        wait_ok_or_err();

        let mut buf = [0; 256];
        for i in buf.iter_mut() {
            *i = ATA_PRIM_BUS0.read();
        }

        Ok(buf)
    }
}

/*pub fn identify(slave: bool) -> Result<Option<[u16; 256]>, ()> {
    unsafe {
        ATA_PRIM_BUS6.write(0xa0 | ((slave as u8) << 4));
        ATA_PRIM_BUS2.write(0x00);
        ATA_PRIM_BUS3.write(0x00);
        ATA_PRIM_BUS4.write(0x00);
        ATA_PRIM_BUS5.write(0x00);
        ATA_PRIM_BUS7.write(0xec);

        if ATA_PRIM_BUS7.read() == 0 { return Ok(None); }

        wait_busy_clear();

        if ATA_PRIM_BUS4.read() == 0 && ATA_PRIM_BUS5.read() == 0 {
            wait_ok_or_err();
        }

        _has_err()?;
        get_next_chunk().map(|a| Some(a))
    }
}*/
