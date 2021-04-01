pub mod keyboard;

/// Describes an object that listens for peripheral events
pub trait IObserver<T> {
    fn update(&self, value: &T);
}

/// Describes a peripheral that sends events to its active observer
pub trait ISubject<'a, T> {
    fn attach(&mut self, observer: &'a dyn IObserver<T>);
    fn notify(&self);
}
