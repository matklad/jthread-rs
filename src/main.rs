use jthread;

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
