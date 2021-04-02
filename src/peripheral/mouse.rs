use super::*;
use ps2_mouse::{Mouse as MouseDevice, MouseState};
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MOUSE_DEVICE: Mutex<MouseDevice> = Mutex::new(MouseDevice::new());
}

pub fn init() {
    MOUSE_DEVICE.lock().init().unwrap();
    MOUSE_DEVICE.lock().set_on_complete(on_complete);
}

fn on_complete(state: MouseState) {
    crate::task::mouse::add_state(state);
}

pub(crate) fn add_packet(packet: u8) {
    let mut mouse = MOUSE_DEVICE.lock();
    mouse.process_packet(packet);
}

pub struct Mouse<'a> {
    observer: Option<&'a dyn IObserver<MouseState>>,
    state: Option<MouseState>,
}
impl<'a> Mouse<'a> {
    pub fn new() -> Self {
        Mouse {
            observer: None,
            state: None,
        }
    }
    pub fn update(&mut self, new_state: MouseState) {
        // if let Some(old_state) = self.state {
        //     if (old_state.get_x() != new_state.get_x()) || (old_state.get_y() != new_state.get_y()) {
        //         self.state = Some(new_state);
        //     } else {
        //         self.state = None;
        //     }
        // } else {
        //     self.state = Some(new_state);
        // }
        self.state = Some(new_state);
    }
}

impl<'a> ISubject<'a, MouseState> for Mouse<'a> {
    fn attach(&mut self, observer: &'a dyn IObserver<MouseState>) {
        self.observer = Some(observer);
    }
    fn notify(&self) {
        if let Some(observer) = self.observer {
            if let Some(state) = self.state {
                observer.update(&state)
            }
        }
    }
}