mod jthread;
mod stop_token;
pub mod io;

pub use crate::{
    jthread::{spawn, JoinHandle},
    stop_token::{StopSource, StopToken, StoppedError},
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
