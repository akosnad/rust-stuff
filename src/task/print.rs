use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use crate::screenbuffer::Screenbuffer;

static PRINT_QUEUE: OnceCell<ArrayQueue<char>> = OnceCell::uninit();
static PRINT_WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_char(character: char) {
    if let Ok(queue) = PRINT_QUEUE.try_get() {
        if let Err(_) = queue.push(character) {
            log::warn!("print queue full; dropping string");
        } else {
            PRINT_WAKER.wake();
        }
    } else { }
}

pub struct CharacterStream {
    _private: (),
}

impl CharacterStream {
    pub fn new() -> Self {
        PRINT_QUEUE.try_init_once(|| ArrayQueue::new(1000))
            .expect("CharacterStream::new should be called once");
        CharacterStream { _private: () }
    }
}

impl Stream for CharacterStream {
    type Item = char;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<char>> {
        let queue = PRINT_QUEUE.try_get().expect("character queue should be initialized by now");

        // fast path
        if let Ok(character) = queue.pop() {
            return Poll::Ready(Some(character));
        }

        PRINT_WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(character) => {
                PRINT_WAKER.take();
                Poll::Ready(Some(character))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_screenbuffer() {
    let mut stream = CharacterStream::new();
    let mut screenbuffer = Screenbuffer::new();

    while let Some(character) = stream.next().await {
        screenbuffer.write_byte(character as u8);
    }
}