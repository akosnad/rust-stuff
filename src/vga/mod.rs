use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;
use vga::colors::{TextModeColor, Color16};

pub mod term;
pub mod writer;

const TEXTMODE_SIZE: (usize, usize) = (80, 25);
const GRAPHICS_SIZE: (usize, usize) = (80, 60);

pub const DEFAULT_COLOR: TextModeColor = TextModeColor::new(Color16::LightGrey, Color16::Blue);