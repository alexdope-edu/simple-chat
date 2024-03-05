use simple_chat::commands::Command;
use simple_chat::commands::CMD_BYE;
use simple_chat::commands::CMD_WHOAMI;
use simple_chat::error::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net;
use std::net::TcpStream;
use std::thread::spawn;
use uuid::uuid;
use uuid::Uuid;

fn handle_connection(connection: TcpStream) -> Result<(), Error> {
    let mut reader = BufReader::new(connection);
    let mut message = String::new();

    loop {
        message.clear();
        // %add_user <Uuid> normal\n%show_users\nпривет\n
        reader.read_line(&mut message).map_err(|e| Error::IO(e))?;

        let cmd = match Command::new(&message) {
            Ok(value) => value,
            Err(error) => {
                println!("{error}");
                continue;
            }
        };
    }
}

struct AcceptedConnection {
    user_id: Option<Uuid>,
    connection_read: BufReader<TcpStream>,
    connection_write: TcpStream,
}

fn main() {
    let server = net::TcpListener::bind("localhost:8889").unwrap();
    let mut all_connections = vec![];

    loop {
        let connection_write = match server.accept() {
            Ok((conn, _)) => conn,
            Err(_) => continue,
        };

        let connection_read = match connection_write.try_clone() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let accepted_connection = AcceptedConnection {
            user_id: None,
            connection_read: BufReader::new(connection_read),
            connection_write,
        };

        all_connections.push(accepted_connection);

        spawn(|| handle_connection(connection).ok());
    }
}

// распарсить команду Login
