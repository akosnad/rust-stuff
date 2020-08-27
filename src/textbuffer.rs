use alloc::vec::Vec;
use vga::colors::{TextModeColor, Color16};
use core::fmt;

#[derive(Debug, Copy, Clone)]
pub struct BufferCharacter {
    pub character: char,
    pub color: TextModeColor,
}

impl BufferCharacter {
    pub fn default_color() -> TextModeColor {
        TextModeColor::new(Color16::LightGrey, Color16::Black)
    }
}

#[derive(Debug, Clone)]
pub struct BufferLine {
    pub chars: Vec<BufferCharacter>,
}

#[derive(Debug)]
pub struct Textbuffer {
    pub lines: Vec<BufferLine>,
    row: usize,
}

impl Textbuffer {
    pub fn new() -> Self {
        let mut textbuffer = Self {
            lines: Vec::new(),
            row: 0,
        };
        textbuffer.lines.push(BufferLine {
            chars: Vec::new(),
        });
        textbuffer
    }

    pub fn flush(&mut self) {
        self.lines.clear();
        self.lines.push(BufferLine {
            chars: Vec::new(),
        });
        self.row = 0;
    }

    pub fn get_lines(&self, from: usize, len: usize) -> Vec<BufferLine> {
        if from + len > self.lines.len() {
            let mut lines = Vec::<BufferLine>::new();
            for i in from..from + len {
                if i < self.lines.len() {
                    lines.push(self.lines[i].clone());
                }
            }
            lines
        } else {
            (&self.lines[from .. from + len]).to_vec()
        }
    }

    pub fn end_coord(&self) -> (usize, usize) {
        (self.row, self.lines[self.row].chars.len())
    }

    pub fn write_char_color(&mut self, character: char, color: TextModeColor) {
        let buffer_character = BufferCharacter {
            character: character,
            color: color,
        };
        self.lines[self.row].chars.push(buffer_character);
    }

    pub fn write_char(&mut self, character: char) {
        self.write_char_color(character, BufferCharacter::default_color())
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.lines.push(BufferLine {
            chars: Vec::new(),
        });
    }

    pub fn write_string(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

}

impl fmt::Write for Textbuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
