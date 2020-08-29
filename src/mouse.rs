use ps2_mouse::{Mouse, MouseState};
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());
}

pub fn init() {
    MOUSE.lock().init().unwrap();
    MOUSE.lock().set_on_complete(on_complete);
}

fn on_complete(state: MouseState) {
    crate::task::term::add_char(0x00 as char);
}