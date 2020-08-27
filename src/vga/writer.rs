use super::*;
use crate::textbuffer::BufferLine;

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

                x86_64::instructions::interrupts::without_interrupts(|| {
                    self.buffer.chars[self.row][self.col].write(*character);
                });
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
    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
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

    pub fn print_textbuffer(&mut self, buf: &[BufferLine]) {
        for row in 0..BUFFER_HEIGHT {
            if row < buf.len() {
                for col in 0..BUFFER_WIDTH {
                    let mut character = b' ';
                    if col < buf[row].chars.len() {
                        character = buf[row].chars[col].character as u8;
                    }
                    let screen_char = ScreenChar {
                        ascii_character: character,
                        color_code: ColorCode::default(),
                    };
                    self.buffer.chars[row][col].write(screen_char);
                }
            } else {
                for col in 0..BUFFER_WIDTH {
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: b' ',
                        color_code: ColorCode::default(),
                    })
                }
            }
        }
    }
}

impl fmt::Write for TextMode {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
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
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[cfg(test)]
use crate::{serial_print, serial_println, println};

#[cfg(test)]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::writer::_print(format_args!($($arg)*)));
}

#[cfg(test)]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}


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
