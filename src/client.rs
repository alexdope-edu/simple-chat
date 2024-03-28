use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
use std::thread::spawn;

fn main() {
    //
    // Устанавливаем TCP-соединение с сервером.
    //

    let connection = TcpStream::connect("localhost:8889").unwrap();

    //
    // Запускаем первый поток, читающий сообщения от сервера.
    //

    let connection_read = connection.try_clone().unwrap(); // клонируем, чтобы отдать в поток?
    let read_thread = spawn(move || read_messages_from_server_write_to_terminal(connection_read));

    //
    // Запускаем второй поток, читающий сообщения из терминала и отравляющий их на сервер.
    //

    spawn(move || read_messages_from_terminal_write_to_server(connection));

    //
    // Блокируем программу до тех пор, пока поток, читающий сообщения от сервера, не завершится.
    // А завершится этот поток только в случае разрыва соединения с сервером.
    //

    read_thread.join().unwrap();
}

fn read_messages_from_server_write_to_terminal(connection: TcpStream) {
    let reader = BufReader::new(connection);
    for message in reader.lines().map(|maybe_message| maybe_message.unwrap()) {
        println!("{}", message);
    }
}

fn read_messages_from_terminal_write_to_server(mut connection: TcpStream) {
    for message in stdin().lines().map(|maybe_message| maybe_message.unwrap()) {
        connection
            .write_all(format!("{message}\n").as_bytes())
            .unwrap();
    }
}
