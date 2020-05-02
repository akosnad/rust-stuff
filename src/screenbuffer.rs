use crate::vga::*;
use core::fmt;
use alloc::string::String;
use conquer_once::spin::OnceCell;

#[derive(Clone, Copy)]
struct Line {
    chars: [ScreenChar; BUFFER_WIDTH],
}

const SCREENBUFFER_SCROLLBACK_ROWS: usize = 1000;

pub static USE_SCREENBUFFER: OnceCell<bool> = OnceCell::uninit();

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
        let mut screenbuffer = Self {
            scrollback: [ Line {
                chars: [ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::default(),
                }; BUFFER_WIDTH],
            }; SCREENBUFFER_SCROLLBACK_ROWS],
            col: 0,
            row: 0,
            scroll_row: 0,
        };
        screenbuffer.scroll_to(0);
        screenbuffer
    }

    fn scroll_to(&mut self, row: usize) {
        let mut writer = WRITER.lock();
        self.scroll_row = if row > SCREENBUFFER_SCROLLBACK_ROWS - BUFFER_HEIGHT { 
            SCREENBUFFER_SCROLLBACK_ROWS - BUFFER_HEIGHT
        } else { row };
        
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

    fn scroll(&mut self, lines: usize, down: bool) {
        let mut new_scroll_row = self.scroll_row;
        if down {
            new_scroll_row += lines;
            if new_scroll_row > SCREENBUFFER_SCROLLBACK_ROWS {
                new_scroll_row = SCREENBUFFER_SCROLLBACK_ROWS;
            }
        } else {
            if new_scroll_row.checked_sub(lines) != None {
                new_scroll_row -= lines;
            } else { new_scroll_row = 0; }
        }
        self.scroll_to(new_scroll_row);
    }

    fn focus_cursor(&mut self) {
        let mut new_scroll_row = 0;
        if self.row > BUFFER_HEIGHT - 1{
            new_scroll_row = self.row - BUFFER_HEIGHT + 1;
        }

        self.scroll_to(new_scroll_row);
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

                if self.row >= self.scroll_row + BUFFER_HEIGHT || self.row < self.scroll_row {
                    self.focus_cursor();
                }

                self.scrollback[self.row].chars[self.col] = ScreenChar {
                    ascii_character: byte,
                    color_code: ColorCode::default(),
                };
                self.col += 1;

                x86_64::instructions::interrupts::without_interrupts(|| {
                    WRITER.lock().write_byte(byte);
                });
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
    if USE_SCREENBUFFER.try_get() == Ok(&true) {
        let mut string = String::new();
        fmt::write(&mut string, args).expect("error converting fmt::Arguments to String");
        for character in string.chars() {
            crate::task::print::add_char(character);
        }
    } else {
        crate::vga::_print(args);
    }
}

#[cfg(not(test))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::screenbuffer::_print(format_args!($($arg)*)));
}

#[cfg(not(test))]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
