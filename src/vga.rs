use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
    pub fn default() -> Self {
        ColorCode::new(Color::LightGray, Color::Black)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
const CURSOR_PORT_CMD: u16 = 0x3d4;
const CURSOR_PORT_DATA: u16 = 0x3d5;
const CURSOR_CMD_SET_POS_X: u16 = 0x0f;
const CURSOR_CMD_SET_POS_Y: u16 = 0x0e;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct TextMode {
    col: usize,
    row: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl TextMode {
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn update_cursor(&mut self) {
        use core::convert::TryInto;
        use x86_64::instructions::port::Port;

        let mut cursor_port_cmd = Port::new(CURSOR_PORT_CMD);
        let mut cursor_port_data = Port::new(CURSOR_PORT_DATA);

        let pos: u16 = (self.row * BUFFER_WIDTH + self.col).try_into().unwrap();
        unsafe {
            cursor_port_cmd.write(CURSOR_CMD_SET_POS_X);
            cursor_port_data.write(pos & 0xff);

            cursor_port_cmd.write(CURSOR_CMD_SET_POS_Y);
            cursor_port_data.write((pos >> 8) & 0xff);
        }
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        // TODO: check x and y ranges
        self.row = y;
        self.col = x;
        self.update_cursor();
    }
    pub fn write_screen_char(&mut self, character: &ScreenChar) {
        match character.ascii_character {
            b'\n' => self.new_line(),
            _ => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.chars[self.row][self.col].write(*character);
                self.col += 1;
                self.update_cursor();
            }
        }
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.write_screen_char(&ScreenChar {
            ascii_character: byte,
            color_code: ColorCode::default(),
        })
    }
    pub fn new_line(&mut self) {
        if self.row == BUFFER_HEIGHT - 1 {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        } else {
            self.row += 1;
        }
        self.col = 0;
        self.update_cursor();
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<TextMode> = Mutex::new(TextMode {
        col: 0,
        row: 0,
        color_code: ColorCode::default(),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[doc(hidden)]
#[cfg(test)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[cfg(test)]
use crate::{serial_print, serial_println, print, println};

#[test_case]
fn test_println_simple() {
    serial_print!("test_println... ");
    println!("test_println_simple output");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });

    serial_println!("[ok]");
}
