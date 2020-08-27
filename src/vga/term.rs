use super::*;
use super::writer::*;
use core::fmt;
use alloc::string::String;
use conquer_once::spin::OnceCell;
use crate::textbuffer::Textbuffer;
use spin::Mutex;
use pc_keyboard::{DecodedKey, KeyCode};

const SCREENBUFFER_SCROLLBACK_ROWS: usize = 1000;

pub static USE_SCREENBUFFER: OnceCell<bool> = OnceCell::uninit();

lazy_static! {
    static ref TERM_BUFFER: Mutex<Textbuffer> = Mutex::new(Textbuffer::new());
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
#[repr(u8)]
pub enum EscapeChar {
    ScrollUp = 1,
    ScrollDown,
    ScrollHome,
    ScrollEnd,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
#[repr(u8)]
pub enum VirtualTerminals {
    KernelLog = 0xF0,
    Console,
}

pub struct Term {
    pub active_term: VirtualTerminals,
    console: Mutex<Textbuffer>,
    col: usize,
    row: usize,
    scroll_row: usize,
}

impl Term {
    pub fn new() -> Self {
        Self {
            active_term: VirtualTerminals::Console,
            console: Mutex::new(Textbuffer::new()),
            col: 0,
            row: 0,
            scroll_row: 0,
        }
    }

    pub fn update_screen(&mut self) {
        let mut writer = WRITER.lock();
        match self.active_term {
            VirtualTerminals::Console => {
                writer.print_textbuffer(&self.console.lock().get_lines(self.scroll_row, BUFFER_HEIGHT))
            },
            VirtualTerminals::KernelLog => {
                writer.print_textbuffer(&crate::klog::LOG_BUFFER.lock().get_lines(self.scroll_row, BUFFER_HEIGHT))
            },
        }
    }

    fn scroll_to(&mut self, row: usize) {
        self.scroll_row = if row > SCREENBUFFER_SCROLLBACK_ROWS - BUFFER_HEIGHT { 
            SCREENBUFFER_SCROLLBACK_ROWS - BUFFER_HEIGHT
        } else { row };
        self.update_screen();
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
        }

        match self.active_term {
            VirtualTerminals::Console => {
                self.console.lock().new_line();
                // WRITER.lock().new_line();
            },
            _ => {}
        }
    }
    
    pub fn change_focus(&mut self, virtual_term: VirtualTerminals) {
        self.active_term = virtual_term;
        self.scroll_row = 0;
        self.update_screen();
    }

    pub fn write_byte(&mut self, byte: u8) {
        match self.active_term {
            VirtualTerminals::Console => {
                match byte {
                    byte if byte == VirtualTerminals::KernelLog as u8 => self.change_focus(VirtualTerminals::KernelLog),
                    byte if byte == VirtualTerminals::Console as u8 => self.change_focus(VirtualTerminals::Console),
                    byte if byte == EscapeChar::ScrollDown as u8 => self.scroll(1, true),
                    byte if byte == EscapeChar::ScrollUp as u8 => self.scroll(1, false),
                    byte if byte == EscapeChar::ScrollHome as u8 => self.scroll_to(0),
                    byte if byte == EscapeChar::ScrollEnd as u8 => self.focus_cursor(),
                    b'\n' => self.new_line(),
                    byte => {
                        if self.col >= BUFFER_WIDTH {
                            self.new_line();
                        }

                        if self.row >= self.scroll_row + BUFFER_HEIGHT || self.row < self.scroll_row {
                            self.focus_cursor();
                        }
                        self.col += 1;
                        self.console.lock().write_char(byte as char);
                        self.update_screen();
                    }
                }
            }
            VirtualTerminals::KernelLog => {
                match byte {
                    byte if byte == VirtualTerminals::KernelLog as u8 => self.change_focus(VirtualTerminals::KernelLog),
                    byte if byte == VirtualTerminals::Console as u8 => self.change_focus(VirtualTerminals::Console),
                    byte if byte == EscapeChar::ScrollDown as u8 => self.scroll(1, true),
                    byte if byte == EscapeChar::ScrollUp as u8 => self.scroll(1, false),
                    byte if byte == EscapeChar::ScrollHome as u8 => self.scroll_to(0),
                    byte if byte == EscapeChar::ScrollEnd as u8 => self.focus_cursor(),
                    _ => {},
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

impl fmt::Write for Term {
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
            crate::task::term::add_char(character);
        }
    } else {
        crate::vga::writer::_print(args);
    }
}

#[cfg(not(test))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::term::_print(format_args!($($arg)*)));
}

#[cfg(not(test))]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
