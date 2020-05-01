use crate::vga::*;
use core::fmt;
use alloc::string::String;

#[derive(Clone, Copy)]
struct Line {
    chars: [ScreenChar; BUFFER_WIDTH],
}

const SCREENBUFFER_SCROLLBACK_ROWS: usize = 1000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
#[repr(u8)]
pub enum EscapeChar {
    ScrollUp = 1,
    ScrollDown,
}

pub struct Screenbuffer {
    scrollback: [Line; SCREENBUFFER_SCROLLBACK_ROWS],
    col: usize,
    row: usize,
    scroll_row: usize,
}

impl Screenbuffer {
    pub fn new() -> Self {
        Self {
            scrollback: [ Line {
                chars: [ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::default(),
                }; BUFFER_WIDTH],
            }; SCREENBUFFER_SCROLLBACK_ROWS],
            col: 0,
            row: 0,
            scroll_row: 0,
        }
    }

    fn scroll(&mut self, lines: usize, down: bool) {
        let mut writer = WRITER.lock();
        if down {
            self.scroll_row += lines;
            if self.scroll_row > SCREENBUFFER_SCROLLBACK_ROWS {
                self.scroll_row = SCREENBUFFER_SCROLLBACK_ROWS;
            }
            writer.new_line();
        } else {
            if self.scroll_row.checked_sub(lines) != None {
                self.scroll_row -= lines;
            } else { self.scroll_row = 0; }
        }

        writer.move_cursor(0, 0);
        for line in self.scrollback[self.scroll_row .. self.scroll_row + BUFFER_HEIGHT].iter() {
            for character in line.chars.iter() {
                writer.write_screen_char(character);
            }
        }
        if self.row.checked_sub(self.scroll_row) != None {
            writer.move_cursor(self.col, self.row - self.scroll_row);
        }
    }

    fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;

        if self.row >= self.scroll_row + BUFFER_HEIGHT {
            self.scroll(1, true);
        } else {
            WRITER.lock().new_line();
        }
    }
    
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte if byte == EscapeChar::ScrollDown as u8 => self.scroll(1, true),
            byte if byte == EscapeChar::ScrollUp as u8 => self.scroll(1, false),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.scrollback[self.row].chars[self.col] = ScreenChar {
                    ascii_character: byte,
                    color_code: ColorCode::default(),
                };
                self.col += 1;

                if self.scroll_row <= self.row && self.row < self.scroll_row + BUFFER_HEIGHT {
                    x86_64::instructions::interrupts::without_interrupts(|| {
                        WRITER.lock().write_byte(byte);
                    });
                }
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
    }
}

impl fmt::Write for Screenbuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let mut string = String::new();
    fmt::write(&mut string, args).expect("error converting fmt::Arguments to String");
    for character in string.chars() {
        crate::task::print::add_char(character);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::screenbuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
