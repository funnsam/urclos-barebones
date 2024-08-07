use core::fmt;
use volatile::Volatile;

pub static mut WRITER: Option<Writer> = None;

pub fn init() {
    unsafe {
        WRITER = Some(
            Writer {
                column_position: 0,
                fg: Color::LightGray,
                bg: Color::Black,
                buffer: core::mem::transmute(0xb8000_usize),
            }
        );
    }
}

pub fn get_writer() -> &'static mut Writer {
    unsafe { WRITER.as_mut().unwrap() }
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl Color {
    pub fn from_ansi(c: u8) -> Option<Self> {
        if c > 7 { return None; }
        Self::from_code(((c & 4) >> 2) | (c & 2) | ((c & 1) << 2))
    }

    pub fn from_code(c: u8) -> Option<Self> {
        if c > 15 { return None; }
        Some(unsafe { core::mem::transmute(c) })
    }

    pub fn bright(&self) -> Option<Self> {
        Self::from_code(*self as u8 + 8)
    }

    pub fn dim(&self) -> Option<Self> {
        Self::from_code(*self as u8 - 8)
    }
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    pub column_position: usize,
    pub fg: Color,
    pub bg: Color,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.get_color_code(),
                });
                self.column_position += 1;
            }
        }
    }

    pub fn fuck(&mut self, byte: u8) {
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: byte,
            color_code: ColorCode::new(Color::White, Color::Blue),
        });
        self.column_position += 1;
    }

    /// Writes the given ASCII string to the buffer.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    pub fn erase_until_eol(&mut self) {
        for col in self.column_position..BUFFER_WIDTH {
            self.buffer.chars[BUFFER_HEIGHT - 1][col].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.get_color_code(),
            });
        }
    }

    pub fn erase_all(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: b' ',
                    color_code: self.get_color_code(),
                });
            }
        }
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.erase_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn get_color_code(&self) -> ColorCode {
        ColorCode::new(self.fg, self.bg)
    }

    /// Clears a row by overwriting it with blank characters.
    pub fn erase_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.get_color_code(),
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
