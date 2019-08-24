use std::{
    future::Future,
    io,
    os::unix::io::{AsRawFd, RawFd},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use libc::c_int;

use crate::{StopToken, StoppedError};

pub struct Stdin {
    fd: RawFd,
    buf: Vec<u8>,
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

impl Stdin {
    pub(crate) fn new() -> Stdin {
        let fd = io::stdin().as_raw_fd();
        unsafe {
            let mut flags = libc::fcntl(fd, libc::F_GETFL);
            assert!(flags != -1);
            flags |= libc::O_NONBLOCK;
            libc::fcntl(fd, libc::F_SETFL, flags);
        }
        Stdin {
            fd,
            buf: Vec::new(),
        }
    }

    pub fn read_line(
        &mut self,
        mut st: &StopToken,
    ) -> Result<Result<String, io::Error>, StoppedError> {
        unsafe {
            let mut fds: [c_int; 2] = [0; 2];
            let res = libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK);
            assert!(res != -1);
            let pipe = Arc::new(Pipe(fds));

            let raw_waker = raw_waker(&pipe);
            let waker = Waker::from_raw(raw_waker);
            let mut context = Context::from_waker(&waker);

            loop {
                let fut = Pin::new(&mut st);
                match Future::poll(fut, &mut context) {
                    Poll::Ready(err) => return Err(err),
                    Poll::Pending => (),
                }

                if let Some(idx) = self.buf.iter().position(|&b| b == b'\n') {
                    let mut buf = self.buf.split_off(idx + 1);
                    std::mem::swap(&mut buf, &mut self.buf);
                    let res = String::from_utf8(buf)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
                    return Ok(res);
                }

                let mut pollfd = [
                    libc::pollfd {
                        fd: self.fd,
                        events: libc::POLLIN,
                        revents: 0,
                    },
                    libc::pollfd {
                        fd: fds[0],
                        events: libc::POLLIN,
                        revents: 0,
                    },
                ];
                let res = libc::poll(pollfd.as_mut_ptr(), 2, -1);
                assert!(res != -1);
                if pollfd[0].revents & libc::POLLIN == libc::POLLIN {
                    self.buf.reserve(4096);
                    let buf = self.buf.as_mut_ptr().add(self.buf.len());
                    let res = libc::read(self.fd, buf as *mut libc::c_void, 4096);
                    if res < 0 {
                        return Ok(Err(io::Error::from_raw_os_error(res as i32)));
                    } else {
                        self.buf.set_len(self.buf.len() + res as usize);
                    }
                }
            }
        }
    }
}

struct Pipe([c_int; 2]);

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            let res = libc::close(self.0[0]);
            assert!(res != -1);
            let res = libc::close(self.0[1]);
            assert!(res != -1);
        }
    }
}

fn raw_waker(pipe: &Arc<Pipe>) -> RawWaker {
    let pipe = Arc::clone(pipe);
    RawWaker::new(Arc::into_raw(pipe) as *const (), &VTABLE)
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |ptr| unsafe {
        let pipe = Arc::from_raw(ptr as *const Pipe);
        let res = raw_waker(&pipe);
        Arc::into_raw(pipe);
        res
    },
    |ptr| unsafe {
        let pipe = Arc::from_raw(ptr as *const Pipe);
        let res = libc::write(pipe.0[1], [92u8].as_ptr() as *const libc::c_void, 1);
        assert!(res != -1);
    },
    |ptr| unsafe {
        let pipe: &Pipe = &*(ptr as *const Pipe);
        let res = libc::write(pipe.0[1], [92u8].as_ptr() as *const libc::c_void, 1);
        assert!(res != -1);
    },
    |_data| (),
);
