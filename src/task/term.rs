use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use crate::vga::term::TERM;

static TERM_QUEUE: OnceCell<ArrayQueue<char>> = OnceCell::uninit();
static TERM_WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_char(character: char) {
    if let Ok(queue) = TERM_QUEUE.try_get() {
        if let Err(_) = queue.push(character) {
            log::warn!("terminal character queue full; dropping character");
        } else {
            TERM_WAKER.wake();
        }
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
        if let Some(character) = queue.pop() {
            return Poll::Ready(Some(character));
        }

        TERM_WAKER.register(&cx.waker());
        match queue.pop() {
            Some(character) => {
                TERM_WAKER.take();
                Poll::Ready(Some(character))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn process_buffer() {
    let mut stream = CharacterStream::new();
    log::debug!("terminal buffer initialized");
    while let Some(character) = stream.next().await {
        let mut term = TERM.lock();
        term.write_byte(character as u8);
    }
}