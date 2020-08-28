use super::*;
use crate::textbuffer::BufferLine;
use vga::writers::{Text80x25, Graphics640x480x16, ScreenCharacter, TextWriter, GraphicsWriter};

enum WriterMode {
    Text,
    Graphics
}

pub struct Writer {
    mode: WriterMode,
    text: Text80x25,
    graphics: Graphics640x480x16,
    col: usize,
    row: usize,
}

impl Writer {
    fn change_mode(&mut self, mode: WriterMode) {
        self.mode = mode;
        match self.mode {
            WriterMode::Text => {
                self.text.set_mode();
            },
            WriterMode::Graphics => {
                self.graphics.set_mode();
            },
        }
    }

    pub fn move_cursor(&mut self, x: usize, y: usize) {
        self.row = y;
        self.col = x;
        match self.mode {
            WriterMode::Text => self.text.set_cursor_position(x, y),
            _ => {}
        }
    }

    fn update_cursor(&mut self) {
        self.move_cursor(self.col, self.row)
    }

    pub fn clear(&mut self) {
        match self.mode {
            WriterMode::Text => { 
                let character = ScreenCharacter::new(b' ', DEFAULT_COLOR);
                self.text.fill_screen(character);
            },
            WriterMode::Graphics => self.graphics.clear_screen(Color16::Blue),
        }
        self.move_cursor(0, 0);
    }

    pub fn write_screen_char(&mut self, character: ScreenCharacter) {
        match character.get_character() {
            b'\n' => self.new_line(),
            _ => {
                match self.mode {
                    WriterMode::Text => {
                        if self.col >= BUFFER_SIZE.0 {
                            self.new_line();
                        }
                        self.text.write_character(self.col, self.row, character);
                    },
                    _ => {},
                }
                self.col += 1;
                self.update_cursor();
            }
        }
    }
    pub fn write_byte(&mut self, byte: u8) {
        self.write_screen_char(ScreenCharacter::new(byte, DEFAULT_COLOR))
    }
    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
    }

    fn clear_row(&mut self, row: usize) {
        let character = ScreenCharacter::new(b' ', DEFAULT_COLOR);
        match self.mode {
            WriterMode::Text => {
                for col in 0..BUFFER_SIZE.0 {
                    self.text.write_character(col, row, character);
                }
            },
            _ => {}
        }
    }

    pub fn new_line(&mut self) {
        match self.mode {
            WriterMode::Text => {
                if self.row == BUFFER_SIZE.1 - 1 {
                    for row in 1..BUFFER_SIZE.1 {
                        for col in 0..BUFFER_SIZE.0 {
                            let character = self.text.read_character(col, row);
                            self.text.write_character(col, row - 1, character);
                        }
                    }
                    self.clear_row(BUFFER_SIZE.1 - 1);
                } else {
                    self.row += 1;
                }
                self.col = 0;
                self.update_cursor();
            },
            _ => {}
        }
    }

    pub fn print_textbuffer(&mut self, buf: &[BufferLine]) {
        match self.mode {
            WriterMode::Text => {
                for row in 0..BUFFER_SIZE.1 {
                    if row < buf.len() {
                        for col in 0..BUFFER_SIZE.0 {
                            let mut character = b' ';
                            if col < buf[row].chars.len() {
                                character = buf[row].chars[col].character as u8;
                            }
                            let screen_char = ScreenCharacter::new(character, DEFAULT_COLOR);
                            self.text.write_character(col, row, screen_char);
                        }
                    } else {
                        for col in 0..BUFFER_SIZE.0 {
                            self.text.write_character(col, row, ScreenCharacter::new(b' ', DEFAULT_COLOR));
                        }
                    }
                }
            },
            _ => {}
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        mode: WriterMode::Text,
        text: Text80x25::new(),
        graphics: Graphics640x480x16::new(),
        col: 0,
        row: 0,
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
