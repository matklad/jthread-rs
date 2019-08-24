use crate::{StopSource, StopToken};

pub struct JoinHandle<T> {
    stop_source: Option<StopSource>,
    inner: Option<std::thread::JoinHandle<T>>,
}

pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce(StopToken) -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let inner;
    let stop_source = StopSource::default();
    let stop_token = stop_source.new_token();
    inner = std::thread::spawn(move || f(stop_token));
    JoinHandle {
        inner: Some(inner),
        stop_source: Some(stop_source),
    }
}

impl<T> JoinHandle<T> {
    pub fn join(mut self) -> std::thread::Result<T> {
        let _ = self.stop_source.take();
        self.inner.take().unwrap().join()
    }
}

// TODO: impl Future for JointHandle<T> ?

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        let _ = self.stop_source.take();
        self.inner.take().unwrap().join().unwrap();
    }
}
