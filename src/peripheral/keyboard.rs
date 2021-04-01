use super::*;
use lazy_static::lazy_static;
use spin::Mutex;
use pc_keyboard::{layouts, HandleControl, Keyboard as KeyboardDevice, ScancodeSet1, DecodedKey};


lazy_static! {
    static ref KEYBOARD: Mutex<KeyboardDevice<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(KeyboardDevice::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
}

pub struct Keyboard<'a> {
    observer: Option<&'a dyn IObserver<DecodedKey>>,
    pub scancode: Option<u8>,
}
impl<'a> Keyboard<'a> {
    pub fn new() -> Self {
        Keyboard {
            observer: None,
            scancode: None
        }
    }
}

impl<'a> ISubject<'a, DecodedKey> for Keyboard<'a> {
    fn attach(&mut self, observer: &'a dyn IObserver<DecodedKey>) {
        self.observer = Some(observer);
    }
    fn notify(&self) {
        if let Some(observer) = self.observer {
            if let Some(scancode) = self.scancode {
                let mut keyboard = KEYBOARD.lock();
                if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                    if let Some(key) = keyboard.process_keyevent(key_event) {
                        observer.update(&key)
                    }
                }
            }
        }
    }
}
