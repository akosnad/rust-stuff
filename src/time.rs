use crossbeam_utils::atomic::AtomicCell;

static TIME: AtomicCell<usize> = AtomicCell::new(0);

/// Called by the timer interrupt handle
/// 
/// Must not block or allocate
pub(crate) fn increment_time() {
    TIME.fetch_add(1);
}

pub fn get() -> usize {
    TIME.load()
}