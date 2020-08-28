use super::*;
use crate::textbuffer::BufferLine;
use vga::writers::{Text80x25, Graphics640x480x16, ScreenCharacter, TextWriter, GraphicsWriter};

#[derive(PartialEq)]
pub enum WriterMode {
    Text,
    Graphics
}

pub struct Writer {
    pub mode: WriterMode,
    text: Text80x25,
    graphics: Graphics640x480x16,
    col: usize,
    row: usize,
}

impl Writer {
    pub fn change_mode(&mut self, mode: WriterMode) {
        self.mode = mode;
        match self.mode {
            WriterMode::Text => {
                self.text = Text80x25::new();
                log::debug!("Switching video mode to: {:?}", self.text);
                self.text.set_mode();
                self.clear();
            },
            WriterMode::Graphics => {
                self.graphics = Graphics640x480x16::new();
                log::debug!("Switching video mode to: {:?}", self.graphics);
                self.graphics.set_mode();
                self.clear();
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
                if self.col >= TEXTMODE_SIZE.0 {
                    self.new_line();
                }
                self.text.write_character(self.col, self.row, character);
                self.col += 1;
                self.update_cursor();
            }
        }
    }
    pub fn write_byte(&mut self, byte: u8) {
        match self.mode {
            WriterMode::Text => {
                self.write_screen_char(ScreenCharacter::new(byte, DEFAULT_COLOR))
            },
            WriterMode::Graphics => {
                match byte {
                    b'\n' => self.new_line(),
                    _ => {
                        if self.col >= GRAPHICS_SIZE.0 {
                            self.new_line();
                        }
                        self.graphics.draw_character(self.col * 8, self.row * 8, byte as char, Color16::White);
                        self.col += 1;
                    }
                }
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
    }

    pub fn clear_row(&mut self, row: usize) {
        let character = ScreenCharacter::new(b' ', DEFAULT_COLOR);
        match self.mode {
            WriterMode::Text => {
                for col in 0..TEXTMODE_SIZE.0 {
                    self.text.write_character(col, row, character);
                }
            },
            WriterMode::Graphics => {
                for y in row * 8..row * 8 + 8 {
                    for x in 0..GRAPHICS_SIZE.0 * 8 - 1 {
                        self.graphics.set_pixel(x, y, Color16::Blue);
                    }
                }
            }
        }
    }

    pub fn new_line(&mut self) {
        match self.mode {
            WriterMode::Text => {
                if self.row == TEXTMODE_SIZE.1 - 1 {
                    for row in 1..TEXTMODE_SIZE.1 {
                        for col in 0..TEXTMODE_SIZE.0 {
                            let character = self.text.read_character(col, row);
                            self.text.write_character(col, row - 1, character);
                        }
                    }
                    self.clear_row(TEXTMODE_SIZE.1 - 1);
                } else {
                    self.row += 1;
                }
                self.col = 0;
                self.update_cursor();
            },
            WriterMode::Graphics => {
                if self.row == GRAPHICS_SIZE.1 - 1 {
                    self.row = 0;
                } else {
                    self.row += 1;
                }
                self.clear_row(self.row);
                self.col = 0;
            }
        }
    }

    pub fn print_textbuffer(&mut self, buf: &[BufferLine]) {
        match self.mode {
            WriterMode::Text => {
                for row in 0..TEXTMODE_SIZE.1 {
                    if row < buf.len() {
                        for col in 0..TEXTMODE_SIZE.0 {
                            let mut character = b' ';
                            if col < buf[row].chars.len() {
                                character = buf[row].chars[col].character as u8;
                            }
                            let screen_char = ScreenCharacter::new(character, DEFAULT_COLOR);
                            self.text.write_character(col, row, screen_char);
                        }
                    } else {
                        for col in 0..TEXTMODE_SIZE.0 {
                            self.text.write_character(col, row, ScreenCharacter::new(b' ', DEFAULT_COLOR));
                        }
                    }
                }
            },
            WriterMode::Graphics => {
                for row in 0..GRAPHICS_SIZE.1 {
                    if row < buf.len() {
                        for col in 0..GRAPHICS_SIZE.0 {
                            let mut character = ' ';
                            if col < buf[row].chars.len() {
                                character = buf[row].chars[col].character;
                            }
                            self.graphics.draw_character(col * 8, row * 8, character, Color16::White);
                        }
                    }
                }
            }
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
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.text.read_character(i, BUFFER_SIZE.1 - 2);
            assert_eq!(char::from(screen_char.get_character()), c);
        }
    });
}
