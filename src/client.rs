use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use std::thread::spawn;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8889").unwrap();
    let mut read_stream = stream.try_clone().unwrap();

    spawn(move || {
        let reader = BufReader::new(&mut read_stream);
        for message in reader.lines().map(|v| v.unwrap()) {
            println!("{message}");
        }
    });

    let handle = spawn(move || {
        for message in stdin().lines().map(|line| line.unwrap()) {
            stream.write_all(format!("{message}\n").as_bytes()).unwrap();
        }
    });

    handle.join().unwrap();
}
