use vga::writers::GraphicsWriter;
use vga::colors::Color16;
use core::convert::TryInto;

pub mod window;

/// A type for describing an pixel's position on-screen
/// 
/// First value is x, second is y
pub type Coord<T> = (T, T);

/// A type for describing an object's size on-screen
///
/// First coordinate is  top right corner, second is bottom left
pub type Geometry<T> = (Coord<T>, Coord<T>);

pub trait GuiDrawable {
    fn fill(&self, writer: &dyn GraphicsWriter<Color16>, geometry: Geometry<isize>, color: Color16) {
        for y in geometry.0.1 + 1..geometry.1.1 {
            writer.draw_line((geometry.0.0 + 1, y), (geometry.1.0 - 1, y), color);
        }
    }

    fn draw_outline(&self, writer: &dyn GraphicsWriter<Color16>, geometry: Geometry<isize>, color: Color16) {
        let mut top_right =     (geometry.1.0, geometry.0.1);
        let mut bottom_right =  (geometry.1.0, geometry.1.1);
        let mut top_left =      (geometry.0.0, geometry.0.1);
        let mut bottom_left =   (geometry.0.0, geometry.1.1);
        for _ in 0..3 {
            writer.draw_line(top_left, top_right, color);
            writer.draw_line(top_left, bottom_left, color);
            writer.draw_line(top_right, bottom_right, color);
            writer.draw_line(bottom_left, bottom_right, color);
            top_right.0 += 1; top_right.1 -= 1;
            top_left.0 -= 1; top_left.1 -= 1;
            bottom_right.0 += 1; bottom_right.1 += 1;
            bottom_left.0 -= 1; bottom_left.1 += 1;
        }
    }

    fn draw_title(&self, writer: &dyn GraphicsWriter<Color16>, geometry: Geometry<isize>, title: &str, color: Color16) {
        let mut x = geometry.0.0 + 3;
        let y = geometry.0.1 + 3;
        for c in (*title).chars() {
            writer.draw_character(x.try_into().unwrap(), y.try_into().unwrap(), c, color);
            x += 8;
        }
    }

    fn draw(&self, writer: &dyn GraphicsWriter<Color16>);
}
