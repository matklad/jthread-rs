use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    sync::Mutex,
    task::{Context, Poll, Waker},
};

pub struct StoppedError;

#[derive(Default)]
pub struct StopSource {
    state: Arc<Mutex<StopState>>,
}

pub struct StopToken {
    state: Arc<Mutex<StopState>>,
}

#[derive(Default)]
struct StopState {
    stop_requested: bool,
    wakers: Vec<Waker>,
}

impl StopToken {
    pub fn check(&self) -> Result<(), StoppedError> {
        if self.state.lock().unwrap().stop_requested {
            Err(StoppedError)
        } else {
            Ok(())
        }
    }
}

impl Future for &'_ StopToken {
    type Output = StoppedError;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<StoppedError> {
        let mut state = self.state.lock().unwrap();
        if state.stop_requested {
            return Poll::Ready(StoppedError);
        }
        state.wakers.push(cx.waker().clone());
        Poll::Pending
    }
}

impl StopSource {
    pub fn new_token(&self) -> StopToken {
        StopToken {
            state: Arc::clone(&self.state),
        }
    }
}

impl Drop for StopSource {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.stop_requested = true;
        for waker in state.wakers.drain(..) {
            waker.wake()
        }
    }
}
