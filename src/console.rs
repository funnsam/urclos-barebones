//! VGA buffer wrapper with (limited) VT100 code supports
use core::fmt;
use x86_64::instructions::port::*;
use crate::vga_buffer::*;

const BUF_LEN: usize = 16;
struct Vt100 {
    in_ctrl: bool,
    control_buf: [u8; BUF_LEN],
    control_len: usize,
}

static mut VT100: Vt100 = Vt100 {
    in_ctrl: false,
    control_buf: [0; BUF_LEN],
    control_len: 0,
};

impl Vt100 {
    fn reject_code(&mut self) {
        self.in_ctrl = false;
        // self.write_string("[rej]");
        for i in 0..self.control_len {
            self.write_byte(self.control_buf[i]);
        }
    }

    fn write_byte(&mut self, b: u8) {
        if self.in_ctrl {
            match b {
                b'C' if self.control_len > 0 => {
                    self.in_ctrl = false;
                    get_writer().column_position += core::str::from_utf8(&self.control_buf[1..self.control_len])
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                },
                b'D' if self.control_len > 0 => {
                    self.in_ctrl = false;
                    get_writer().column_position -= core::str::from_utf8(&self.control_buf[1..self.control_len])
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                },
                b'K' if self.control_len == 1 && self.control_buf[0] == b'[' => {
                    self.in_ctrl = false;
                    get_writer().erase_until_eol();
                },
                b'J' if self.control_len == 2 && self.control_buf[0] == b'[' => {
                    self.in_ctrl = false;
                    let wr = get_writer();

                    match self.control_buf[1] {
                        b'2' => wr.erase_all(),
                        _ => self.reject_code(),
                    }
                },
                b'm' if self.control_len > 1 && self.control_buf[0] == b'[' => {
                    self.in_ctrl = false;

                    let wr = get_writer();
                    let mut nuh_uh = false;

                    for c in self.control_buf[1..self.control_len].split(|c| *c == b';') {
                        match (c[0], c.len()) {
                            (b'0', 1) => {
                                wr.fg = Color::LightGray;
                                wr.bg = Color::Black;
                            },
                            (b'1', 1) => { wr.fg.bright().map(|c| wr.fg = c); },
                            (b'2', 1) => { wr.fg.dim().map(|c| wr.fg = c); },
                            (b'3', 2) => wr.fg = match c[1] {
                                b'0'..=b'7' => Color::from_ansi(c[1] - b'0').unwrap(),
                                b'9' => Color::LightGray,
                                _ => { nuh_uh = true; break; },
                            },
                            (b'4', 2) => wr.bg = match c[1] {
                                b'0'..=b'7' => Color::from_ansi(c[1] - b'0').unwrap(),
                                b'9' => Color::LightGray,
                                _ => { nuh_uh = true; break; },
                            },
                            _ => { nuh_uh = true; break; },
                        }
                    }

                    if nuh_uh { self.reject_code(); }
                },
                _ => {
                    self.control_buf[self.control_len] = b;
                    self.control_len += 1;
                    if self.control_len >= BUF_LEN {
                        self.reject_code();
                    }
                },
            }
        } else {
            match b {
                0x07 => {},
                0x1b => {
                    self.in_ctrl = true;
                    self.control_len = 0;
                },
                _ => get_writer().write_byte(b),
            }
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
        unsafe {
            VT100.write_fmt(args).unwrap();

            let pos = 24 * BUFFER_WIDTH + get_writer().column_position;
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
