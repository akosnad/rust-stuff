use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use crate::vga::term::{Textbuffer, USE_SCREENBUFFER};

static TERM_QUEUE: OnceCell<ArrayQueue<char>> = OnceCell::uninit();
static TERM_WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_char(character: char) {
    if let Ok(queue) = TERM_QUEUE.try_get() {
        if let Err(_) = queue.push(character) {
            log::warn!("terminal character queue full; dropping character");
        } else {
            TERM_WAKER.wake();
        }
    } else {
        log::warn!("terminal character queue uninitialized")
    }
}

pub struct CharacterStream {
    _private: (),
}

impl CharacterStream {
    pub fn new() -> Self {
        TERM_QUEUE.try_init_once(|| ArrayQueue::new(1000))
            .expect("CharacterStream::new should be called once");
        CharacterStream { _private: () }
    }
}

impl Stream for CharacterStream {
    type Item = char;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<char>> {
        let queue = TERM_QUEUE.try_get().expect("terminal character queue should be initialized by now");

        // fast path
        if let Ok(character) = queue.pop() {
            return Poll::Ready(Some(character));
        }

        TERM_WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(character) => {
                TERM_WAKER.take();
                Poll::Ready(Some(character))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_screenbuffer() {
    let mut stream = CharacterStream::new();
    let mut textbuffer = Textbuffer::new();
    USE_SCREENBUFFER.try_init_once(|| true).expect("USE_SCREENBUFFER should be initialized once");
    log::debug!("screenbuffer initialized");
    while let Some(character) = stream.next().await {
        textbuffer.write_byte(character as u8);
    }
}