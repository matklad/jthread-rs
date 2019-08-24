# jthread

A proof-of-concept implementation of C++'s [jthread](https://github.com/josuttis/jthread) for Rust.

The main difference is that instead of using `std::stop_callback`, `StopToken`
implements `std::future::Future`. This is enough to make syscalls like `read(2)`
cancellable, while maintaining a synchronous API.


```rust
use jthread;

// Note: this is synchronous, blocking code.
fn main() {
    let reader = jthread::spawn(|stop_token| {
        let mut stdin = jthread::io::stdin();

        // `stop_token` allows to break free from a blocking read call
        while let Ok(line) = stdin.read_line(&stop_token) {
            let line = line.unwrap();
            println!("> {:?}", line)
        }
        println!("exiting") // this will be printed to the screen
    });

    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(reader); // cancels and joins the reader thread
}
```


Internally, `read_line` uses `poll` to select between reading from the file
descriptor and the cancelled notification.
