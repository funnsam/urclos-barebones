//! VGA buffer wrapper with (limited) VT100 code supports
use core::fmt::{self, Write};
use spin::Mutex;
use x86_64::instructions::port::*;
use crate::vga_buffer::*;

const BUF_LEN: usize = 5;
struct Vt100 {
    in_ctrl: bool,
    control_buf: [u8; BUF_LEN],
    control_len: usize,
}

lazy_static::lazy_static! {
    static ref VT100: Mutex<Vt100> = Mutex::new(Vt100 {
        in_ctrl: false,
        control_buf: [0; BUF_LEN],
        control_len: 0,
    });
}

impl Vt100 {
    fn reject_code(&mut self) {
        self.in_ctrl = false;
        self.write_string("[rej]");
        for i in 0..self.control_len {
            self.write_byte(self.control_buf[i]);
        }
    }

    fn write_byte(&mut self, b: u8) {
        if self.in_ctrl {
            match b {
                b'C' if self.control_len > 0 => {
                    self.in_ctrl = false;
                    WRITER.lock().column_position += core::str::from_utf8(&self.control_buf[1..self.control_len])
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                },
                b'D' if self.control_len > 0 => {
                    self.in_ctrl = false;
                    WRITER.lock().column_position -= core::str::from_utf8(&self.control_buf[1..self.control_len])
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                },
                b'K' if self.control_len == 1 => {
                    self.in_ctrl = false;
                    WRITER.lock().erase_until_eol();
                },
                b'm' => {
                    self.in_ctrl = false;

                    if self.control_len < 2 || self.control_buf[0] != b'[' {
                        self.reject_code();
                        return;
                    }

                    let mut wr = WRITER.lock();
                    match (self.control_buf[1], self.control_len) {
                        (b'0', 2) => {
                            wr.set_foreground(Color::LightGray);
                            wr.set_background(Color::Black);
                        },
                        (b'1', 2) => {},
                        (b'3', 3) => wr.set_foreground(match self.control_buf[2] {
                            b'0' => Color::Black,
                            b'1' => Color::Red,
                            b'2' => Color::Green,
                            b'3' => Color::Yellow,
                            b'4' => Color::Blue,
                            b'5' => Color::Magenta,
                            b'6' => Color::Cyan,
                            b'7' => Color::LightGray,
                            b'9' => Color::LightGray,
                            _ => { self.reject_code(); return; }
                        }),
                        (b'4', 3) => wr.set_background(match self.control_buf[2] {
                            b'0' => Color::Black,
                            b'1' => Color::Red,
                            b'2' => Color::Green,
                            b'3' => Color::Yellow,
                            b'4' => Color::Blue,
                            b'5' => Color::Magenta,
                            b'6' => Color::Cyan,
                            b'7' => Color::LightGray,
                            b'9' => Color::LightGray,
                            _ => { self.reject_code(); return; }
                        }),
                        _ => self.reject_code(),
                    }
                },
                _ => {
                    self.control_buf[self.control_len] = b;
                    self.control_len += 1;
                    if self.control_len >= BUF_LEN {
                        self.reject_code();
                    }
                },
            }
        } else if b == 0x1b {
            self.in_ctrl = true;
            self.control_len = 0;
        } else {
            WRITER.lock().write_byte(b);
        }
    }

    fn write_bytes(&mut self, s: &[u8]) {
        for byte in s.iter() {
            self.write_byte(*byte);
        }
    }

    fn write_string(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }
}

impl fmt::Write for Vt100 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        // WRITER.lock().write_fmt(args).unwrap();
        VT100.lock().write_fmt(args).unwrap();

        unsafe {
            let pos = 24 * BUFFER_WIDTH + WRITER.lock().column_position;
            PortWrite::write_to_port(0x3d4, 0x0f_u8);
            PortWrite::write_to_port(0x3d5, pos as u8);
            PortWrite::write_to_port(0x3d4, 0x0e_u8);
            PortWrite::write_to_port(0x3d5, (pos >> 8) as u8);
        }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
