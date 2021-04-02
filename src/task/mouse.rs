use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use crate::peripheral::{ISubject, mouse::Mouse};
use ps2_mouse::MouseState;

static STATE_QUEUE: OnceCell<ArrayQueue<MouseState>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub(crate) fn add_state(state: MouseState) {
    if let Ok(queue) = STATE_QUEUE.try_get() {
        if let Err(_) = queue.push(state) {
            log::warn!("mouse state queue ful; dropping input")
        } else {
            WAKER.wake();
        }
    }
}

pub struct StateStream {
    _private: (),
}

impl StateStream {
    pub fn new() -> Self {
        STATE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("StateStream::new should be called once");
        StateStream { _private: () }
    }
}

impl Stream for StateStream {
    type Item = MouseState;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<MouseState>> {
        let queue = STATE_QUEUE.try_get().expect("state queue should be initialized by now");

        // fast path
        if let Some(state) = queue.pop() {
            return Poll::Ready(Some(state));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(state) => {
                WAKER.take();
                Poll::Ready(Some(state))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn process_states(mut mouse_subject: Mouse<'_>) {
    let mut states = StateStream::new();
    log::debug!("mouse state stream initialized");
    while let Some(state) = states.next().await {
        mouse_subject.update(state);
        mouse_subject.notify();
    }
}