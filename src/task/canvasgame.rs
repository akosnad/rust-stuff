use canvasgame_rust::{engine::bare::BareEngine, world::World, world::Entity, world::Coord};
use futures_util::stream::{Stream, StreamExt};
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::task::AtomicWaker;
use lazy_static::lazy_static;
use spin::Mutex;

static WAKER: AtomicWaker = AtomicWaker::new();

lazy_static! {
    static ref LAST_FIRE: Mutex<usize> = Mutex::new(0);
}

pub(crate) fn next() {
    WAKER.wake();
}

struct Interval {
    duration: usize,
}
impl Interval {
    pub fn new(duration: usize) -> Self {
        Self {
            duration: duration,
        }
    }
}
impl Stream for Interval {
    type Item = usize;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<usize>> {
        let now = crate::time::get();
        let mut last_fire = LAST_FIRE.lock();
        let delta = now - *last_fire;
        //log::trace!("now: {}, delta: {}, dur: {}, last: {}", now, delta, self.duration, last_fire);
        WAKER.register(&cx.waker());
        if delta >= self.duration {
            WAKER.take();
            *last_fire = now;
            return Poll::Ready(Some(delta));
        }
        return Poll::Pending;
    }
}

pub async fn run() {
    use core::convert::TryInto;

    //let buf = crate::vga::writer::WRITER.lock().graphics.get_frame_buffer();
    let buf: *mut u8 = 0xa0000 as *mut u8;
    let mut c = |x: usize, y: usize, r: u8, g: u8, b: u8| {
        unsafe {
            *buf.offset((y / 2 * 320 / 2 + x / 4).try_into().unwrap()) = (r >> 5) + ((g >> 5) << 3) + ((b >> 6) << 6);
        }
    };

    let mut world = World::new();
    let mut e = Entity::new();
    e.pos = Coord {
        x: { 40.0 },
        y: { 35.0 },
        z: { 15.0 }
    };
    world.entities.push(e);
    let mut engine = BareEngine::new(world, 320, 240, &mut c);
    let mut interval = Interval::new(20);
    //let test: Vec<u8> = vec![255; 100];
    loop {
        if let Some(_) = interval.next().await {
            engine.tick();
            let writer = crate::vga::writer::WRITER.lock();
            if writer.mode == crate::vga::writer::WriterMode::Game {
                engine.render();
            }
        }
    }
}
