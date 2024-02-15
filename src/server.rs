use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net;
use std::thread::spawn;

const CMD_WHOAMI: &str = "whoami";
const CMD_BYE: &str = "bye";

fn main() {
    let server = net::TcpListener::bind("localhost:8889").unwrap();

    for mut stream in server.incoming().map(|v| v.unwrap()) {
        spawn(move || {
            let welcome_message =
                format!("Welcome to the chat server!\nCommands: {CMD_WHOAMI}, {CMD_BYE}\n");

            stream.write_all(welcome_message.as_bytes()).unwrap();

            let mut reader = BufReader::new(&mut stream);
            let mut cmd = String::new();

            loop {
                reader.read_line(&mut cmd).unwrap();
                match cmd.trim() {
                    CMD_WHOAMI => println!("please introduce yourself"),
                    CMD_BYE => return,
                    _ => continue,
                }
            }

            // for line in reader.lines().map(|maybe_line| maybe_line.unwrap()) {
            //     println!("{line}");
            // }
        });
    }
}
