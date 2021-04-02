use super::*;
use super::writer::*;
use core::fmt;
use alloc::string::String;
use crate::textbuffer::Textbuffer;
use spin::Mutex;
use num_enum::FromPrimitive;
use crate::gui::{window::Window, GuiDrawable};
use crate::peripheral::{IObserver};
use ps2_mouse::MouseState;
use pc_keyboard::{DecodedKey, KeyCode};

lazy_static! {
    static ref TERM_BUFFER: Mutex<Textbuffer> = Mutex::new(Textbuffer::new());
}

lazy_static! {
    pub static ref TERM: Mutex<Term> = Mutex::new(Term::new());
}

lazy_static! {
    pub static ref TERM_INPUT: TermInput = TermInput { _private: () };
}

lazy_static! {
    static ref TEST_WINDOW: Window<'static> = Window::new();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, FromPrimitive)]
#[repr(u8)]
pub enum EscapeChar {
    #[num_enum(default)]
    Null = 0,
    ScrollUp,
    ScrollDown,
    ScrollHome,
    ScrollEnd,
    ScrollRight,
    ScrollLeft,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug, FromPrimitive)]
#[repr(u8)]
pub enum VirtualTerminals {
    KernelLog = 0xF0,
    Console,
    GUI,
    ScreenTest,
    #[num_enum(default)]
    Unknown,
}

pub struct Term {
    pub active_term: VirtualTerminals,
    console: Mutex<Textbuffer>,
    col: usize,
    row: usize,
    scroll_row: usize,
    scroll_col: usize,
    mouse_pos: (usize, usize),
}

impl Term {
    pub fn new() -> Self {
        Self {
            active_term: VirtualTerminals::Console,
            console: Mutex::new(Textbuffer::new()),
            col: 0,
            row: 0,
            scroll_row: 0,
            scroll_col: 0,
            mouse_pos: (0, 0),
        }
    }

    pub fn update_screen(&mut self) {
        let mut writer = WRITER.lock();
        if writer.mode != WriterMode::Graphics && self.active_term == VirtualTerminals::GUI {
            writer.change_mode(WriterMode::Graphics);
        } else if writer.mode != WriterMode::Text && self.active_term != VirtualTerminals::GUI {
            writer.change_mode(WriterMode::Text);
        }

        let mut lines: alloc::vec::Vec<crate::textbuffer::BufferLine>;
        match self.active_term {
            VirtualTerminals::Console => {
                lines = self.console.lock().get_lines(self.scroll_row, TEXTMODE_SIZE.1);
            },
            VirtualTerminals::KernelLog => {
                lines = crate::klog::LOG_BUFFER.lock().get_lines(self.scroll_row, TEXTMODE_SIZE.1);
            },
            VirtualTerminals::GUI => {
                TEST_WINDOW.draw(writer.get_graphics_writer());
                return;
            }
            VirtualTerminals::ScreenTest => {
                writer.clear();
                let mut string = String::new();
                fmt::write(&mut string, format_args!(
                    "Commit hash: {}\nCommit date: {}\n",
                    env!("GIT_HASH"),
                    env!("GIT_HASH_DATE")
                )).ok();
                writer.write_string(&string);
                for i in 0x0..0xff {
                    if i % 0xf == 0 {
                        writer.new_line();
                    }
                    writer.write_byte(i);
                }
                return;
            },
            _ => { return; }
        }
        if self.scroll_col > 0 {
            for line in lines.iter_mut() {
                if self.scroll_col < line.chars.len() {
                    line.chars = line.chars.split_off(self.scroll_col);
                } else { line.chars.clear(); }
            }
        }
        writer.print_textbuffer(&lines);
        let (x, y) = self.get_cursor();
        writer.move_cursor(x, y);
    }

    pub fn update_mouse(&mut self, x: isize, mut y: isize) {
        match self.active_term {
            VirtualTerminals::GUI => {
                if x < 0 {
                    if self.mouse_pos.0.checked_sub(x.abs() as usize) == None {
                        self.mouse_pos.0 = 0;
                    } else {
                        self.mouse_pos.0 -= x.abs() as usize;
                    }
                } else if self.mouse_pos.0.checked_add(x as usize) == None || self.mouse_pos.0 >= (GRAPHICS_SIZE.0 * 8) - 1 {
                    self.mouse_pos.0 = GRAPHICS_SIZE.0 * 8 - 1;
                } else {
                    self.mouse_pos.0 += x as usize;
                }

                y = -y;
                if y < 0 {
                    if self.mouse_pos.1.checked_sub(y.abs() as usize) == None {
                        self.mouse_pos.1 = 0;
                    } else {
                        self.mouse_pos.1 -= y.abs() as usize;
                    }
                } else if self.mouse_pos.1.checked_add(y as usize) == None || self.mouse_pos.1 >= (GRAPHICS_SIZE.1 * 8) - 1 {
                    self.mouse_pos.1 = GRAPHICS_SIZE.1 * 8 - 1;
                } else {
                    self.mouse_pos.1 += y as usize;
                }

                let mut writer = WRITER.lock();
                writer.draw_mouse(self.mouse_pos.0, self.mouse_pos.1);
            }
            _ => {}
        }
    }

    fn get_cursor(&self) -> (usize, usize) {
        let mut row;
        if self.row.checked_sub(self.scroll_row) != None {
            row = self.row - self.scroll_row;
        } else if self.scroll_row + TEXTMODE_SIZE.1 < self.row {
            row = self.row;
        } else {
            row = TEXTMODE_SIZE.1 + 1; // Offscreen
        }
        let col;
        if self.col.checked_sub(self.scroll_col) != None {
            col = self.col - self.scroll_col;
        } else if self.scroll_col + TEXTMODE_SIZE.0 < self.col {
            col = 0;
            row = TEXTMODE_SIZE.1 +1; // Offscreen
        } else {
            col = 0;
            row = TEXTMODE_SIZE.1 + 1; // Offscreen
        }
        (col, row)
    }

    fn scroll_to(&mut self, row: usize) {
        self.scroll_row = row;
        self.update_screen();
    }

    fn scroll(&mut self, lines: usize, down: bool) {
        let mut new_scroll_row = self.scroll_row;
        if down {
            new_scroll_row += lines;
        } else {
            if new_scroll_row.checked_sub(lines) != None {
                new_scroll_row -= lines;
            } else { new_scroll_row = 0; }
        }
        self.scroll_to(new_scroll_row);
    }

    fn scroll_to_vert(&mut self, col: usize) {
        self.scroll_col = col;
        self.update_screen();
    }

    fn scroll_vert(&mut self, columns: usize, right: bool) {
        let mut new_scroll_col = self.scroll_col;
        if right {
            new_scroll_col += columns;
        } else {
            if new_scroll_col.checked_sub(columns) != None {
                new_scroll_col -= columns;
            } else { new_scroll_col = 0; }
        }
        self.scroll_to_vert(new_scroll_col);
    }

    fn focus_cursor(&mut self) {
        let mut new_scroll_row = 0;
        if self.row > TEXTMODE_SIZE.1 - 1 {
            new_scroll_row = self.row - TEXTMODE_SIZE.1 + 1;
        }
        let mut new_scroll_col = 0;
        if self.col > TEXTMODE_SIZE.0 - 2 {
            new_scroll_col = self.col - TEXTMODE_SIZE.0 + 2;
        }
        self.scroll_to(new_scroll_row);
        self.scroll_to_vert(new_scroll_col);
    }

    fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;

        if self.row >= self.scroll_row + TEXTMODE_SIZE.1 {
            self.scroll(1, true);
        }
        self.scroll_to_vert(0);

        match self.active_term {
            VirtualTerminals::Console => {
                self.console.lock().new_line();
                let (x, y) = self.get_cursor();
                WRITER.lock().move_cursor(x, y);
            },
            _ => {}
        }
    }
    
    pub fn change_focus(&mut self, virtual_term: VirtualTerminals) {
        self.active_term = virtual_term;
        match self.active_term {
            VirtualTerminals::KernelLog => {
                let (row, col) = crate::klog::LOG_BUFFER.lock().end_coord();
                self.row = row;
                self.col = col;
                self.focus_cursor();
            },
            VirtualTerminals::Console => {
                let (row, col) = self.console.lock().end_coord();
                self.row = row;
                self.col = col;
                self.focus_cursor();
            },
            VirtualTerminals::GUI => {
                self.update_screen();
            }
            VirtualTerminals::ScreenTest => {
                self.col = 0;
                self.row = 0;
                self.scroll_row = 0;
                self.update_screen();
            },
            _ => {}
        }
    }

    fn handle_escape_char(&mut self, escape_char: EscapeChar) {
        match escape_char {
            EscapeChar::ScrollDown =>   self.scroll(1, true),
            EscapeChar::ScrollUp =>     self.scroll(1, false),
            EscapeChar::ScrollHome =>   { self.scroll_to(0); self.scroll_to_vert(0); },
            EscapeChar::ScrollEnd =>    self.focus_cursor(),
            EscapeChar::ScrollRight =>  self.scroll_vert(10, true),
            EscapeChar::ScrollLeft =>   self.scroll_vert(10, false),
            _ => {}
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match self.active_term {
            VirtualTerminals::Console => {
                match byte {
                    byte if VirtualTerminals::from(byte) != VirtualTerminals::Unknown => self.change_focus(VirtualTerminals::from(byte)),
                    byte if EscapeChar::from(byte) != EscapeChar::Null => self.handle_escape_char(EscapeChar::from(byte)),
                    byte if byte == 0x00 => self.update_screen(),
                    byte => {
                        if self.row >= self.scroll_row + TEXTMODE_SIZE.1
                            || self.row < self.scroll_row
                            || self.col >= TEXTMODE_SIZE.0
                            || self.col < self.scroll_col {
                            self.focus_cursor();
                        }
                        if byte == b'\n' {
                            self.new_line();
                        } else {
                            self.col += 1;
                            self.console.lock().write_char(byte as char);
                        }
                        self.update_screen();
                    }
                }
            }
            VirtualTerminals::KernelLog => {
                match byte {
                    byte if VirtualTerminals::from(byte) != VirtualTerminals::Unknown => self.change_focus(VirtualTerminals::from(byte)),
                    byte if EscapeChar::from(byte) != EscapeChar::Null => self.handle_escape_char(EscapeChar::from(byte)),
                    byte if byte == 0x08 => log::trace!("Backspace"),
                    byte if byte == 0x00 => self.update_screen(),
                    _ => {},
                }
            },
            VirtualTerminals::GUI => {
                match byte {
                    byte if VirtualTerminals::from(byte) != VirtualTerminals::Unknown => self.change_focus(VirtualTerminals::from(byte)),
                    byte if byte == 0x00 => self.update_screen(),
                    _ => {}
                }
            }
            VirtualTerminals::ScreenTest => {
                match byte {
                    byte if VirtualTerminals::from(byte) != VirtualTerminals::Unknown => self.change_focus(VirtualTerminals::from(byte)),
                    _ => {}
                }
        },
            _ => {}
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_byte(c as u8);
        }
    }
}

pub struct TermInput {
    _private: (),
}
impl IObserver<DecodedKey> for TermInput {
    fn update(&self, value: &DecodedKey) {
        use crate::task::term::add_char;

        match *value {
            DecodedKey::RawKey(KeyCode::ArrowUp) => add_char(EscapeChar::ScrollUp as u8 as char),
            DecodedKey::RawKey(KeyCode::ArrowDown) => add_char(EscapeChar::ScrollDown as u8 as char),
            DecodedKey::RawKey(KeyCode::Home) => add_char(EscapeChar::ScrollHome as u8 as char),
            DecodedKey::RawKey(KeyCode::End) => add_char(EscapeChar::ScrollEnd as u8 as char),
            DecodedKey::RawKey(KeyCode::ArrowRight) => add_char(EscapeChar::ScrollRight as u8 as char),
            DecodedKey::RawKey(KeyCode::ArrowLeft) => add_char(EscapeChar::ScrollLeft as u8 as char),
            DecodedKey::RawKey(KeyCode::F1) => add_char(VirtualTerminals::KernelLog as u8 as char),
            DecodedKey::RawKey(KeyCode::F2) => add_char(VirtualTerminals::Console as u8 as char),
            DecodedKey::RawKey(KeyCode::F3) => add_char(VirtualTerminals::GUI as u8 as char),
            DecodedKey::RawKey(KeyCode::F12) => add_char(VirtualTerminals::ScreenTest as u8 as char),
            DecodedKey::Unicode(character) => add_char(character),
            DecodedKey::RawKey(key) => add_char(key as u8 as char),
        }
    }
}

impl IObserver<MouseState> for TermInput {
    fn update(&self, value: &MouseState) {
        let mut term = TERM.lock();
        term.update_mouse(value.get_x().into(), value.get_y().into());
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
    let mut string = String::new();
    fmt::write(&mut string, args).expect("error converting fmt::Arguments to String");
    for character in string.chars() {
        crate::task::term::add_char(character);
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
