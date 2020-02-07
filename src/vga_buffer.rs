use alloc::vec::Vec;
use core::fmt;
use core::fmt::Write;
use core::ops::{Index, IndexMut};
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
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const CURSOR_PORT_CMD: u16 = 0x3d4;
const CURSOR_PORT_DATA: u16 = 0x3d5;
const CURSOR_CMD_SET_POS_X: u16 = 0x0f;
const CURSOR_CMD_SET_POS_Y: u16 = 0x0e;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub trait Writer: fmt::Write {
    fn new_line(&mut self);
    fn write_byte(&mut self, byte: u8);
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }
    fn scroll_up(&mut self, _lines: usize) {}
    fn scroll_down(&mut self, _lines: usize) {}
}

pub struct PlainVGA {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl PlainVGA {
    fn move_cursor(x: usize, y: usize) {
        use core::convert::TryInto;
        use x86_64::instructions::port::Port;

        let mut cursor_port_cmd = Port::new(CURSOR_PORT_CMD);
        let mut cursor_port_data = Port::new(CURSOR_PORT_DATA);

        let pos: u16 = (y * BUFFER_WIDTH + x).try_into().unwrap();
        unsafe {
            cursor_port_cmd.write(CURSOR_CMD_SET_POS_X);
            cursor_port_data.write(pos & 0xff);

            cursor_port_cmd.write(CURSOR_CMD_SET_POS_Y);
            cursor_port_data.write((pos >> 8) & 0xff);
        }
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Writer for PlainVGA {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
                PlainVGA::move_cursor(self.column_position, BUFFER_HEIGHT - 1);
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
        PlainVGA::move_cursor(0, BUFFER_HEIGHT - 1);
    }
}

impl fmt::Write for PlainVGA {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub struct ScrollbackVGA {
    scrollback: Vec<Vec<ScreenChar>>,
    scroll_row: usize,
    column_position: usize,
    row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl ScrollbackVGA {
    fn update_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            let line = self.scrollback.index(row);
            for col in 0..BUFFER_WIDTH {
                let character = line.index(col);
                self.buffer.chars[row][col].write(*character);
            }
        }
    }

    fn scroll_down(&mut self, lines: usize) {
        self.scroll_row += lines;
        self.update_screen();
    }

    fn scroll_up(&mut self, lines: usize) {
        self.scroll_row -= lines;
        // if self.scroll_row < 0 {
        //     self.scroll_row = 0;
        // }
        self.update_screen();
    }
}

impl Writer for ScrollbackVGA {
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position - self.scroll_row;
                let col = self.column_position;
                let current_line = self.scrollback.index_mut(self.row_position);

                let color_code = self.color_code;
                current_line.push(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
                if row <= BUFFER_HEIGHT {
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: byte,
                        color_code: color_code,
                    });
                    PlainVGA::move_cursor(col + 1, row);
                }
            }
        }
    }

    fn new_line(&mut self) {
        self.scrollback.push(Vec::with_capacity(BUFFER_WIDTH));
        self.row_position += 1;
        self.column_position = 0;
        self.scroll_down(1);
    }
}

impl fmt::Write for ScrollbackVGA {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref PLAINVGA: Mutex<PlainVGA> = Mutex::new(PlainVGA {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

lazy_static! {
    pub static ref VGABUFFER: dyn Writer = PLAINVGA;
}

pub fn init_scrollback() {
    let ref mut writer = ScrollbackVGA {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::LightGray, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        scroll_row: 0,
        scrollback: Vec::new(),
    };
    writer.scrollback.push(Vec::with_capacity(BUFFER_WIDTH));

    let mut vga_buffer = Mutex::new(VGABuffer { writer: writer });
    VGABUFFER = vga_buffer;
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        VGABUFFER.lock().writer.write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[cfg(test)]
use crate::{serial_print, serial_println};

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
