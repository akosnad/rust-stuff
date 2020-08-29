use super::*;
use vga::writers::{Graphics640x480x16};
use vga::colors::Color16;

pub struct Window<'a> {
    geometry: Geometry<isize>,
    title: &'a str,
}

impl Window<'_> {
    pub fn new() -> Self {
        Self {
            geometry: (
                (80, 60),
                (540, 420)
            ),
            title: &"Test window",
        }
    }
}

impl GuiDrawable for Window<'_> {
    fn draw(&self, writer: &Graphics640x480x16) {
        self.draw_outline(writer, self.geometry, Color16::LightGrey);
        self.fill(writer, self.geometry, Color16::DarkGrey);
        self.draw_title(writer, self.geometry, self.title, Color16::White);
    }
}