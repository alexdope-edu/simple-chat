use simple_chat::commands::Command;
use simple_chat::commands::Message;
use simple_chat::commands::CMD_BYE;
use simple_chat::commands::CMD_WHOAMI;
use simple_chat::error::Error;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::spawn;
use uuid::Uuid;

type AcceptedConnections = Arc<Mutex<HashMap<Uuid, AcceptedConnection>>>;

struct AcceptedConnection {
    connection: TcpStream,
    user_id: Option<String>,
}

fn main() {
    //
    // Слушаем порт 8889 на IP-адресе localhost (локальный адрес).
    //

    let server = net::TcpListener::bind("localhost:8889").unwrap();

    //
    // Создаем контейнер для размещения всех принятых соединений.
    //

    let accepted_connections = Arc::new(Mutex::new(HashMap::new()));

    //
    // Бесконечно принимаем новые соединения от клиентов.
    //

    loop {
        //
        // Блокируем программу на вызове server.accept() до тех пор, пока какой-то из клиентов
        // не попытается установить с нами TCP-соединение.
        //

        let connection = match server.accept() {
            Err(_) => continue,
            Ok((conn, _)) => conn,
        };

        //
        // Генерируем уникальный идентификатор подключения.
        //

        let connection_id = Uuid::new_v4();

        //
        // Клонируем принятое TCP-соединение.
        //

        let connection_clone = match connection.try_clone() {
            Err(_) => continue,
            Ok(value) => value,
        };

        //
        // Добавляем принятое подключение в общий список подключений.
        //

        accepted_connections.lock().unwrap().insert(
            connection_id,
            AcceptedConnection {
                connection: connection_clone,
                user_id: None.into(),
            },
        );

        //
        // Создаем новый поток, где будет обрабатываться принятое соединение.
        // Поскольку перед closure, которую передаем в spawn, стоит ключевое слово move,
        // closure принимает владение всеми переменными, которые используются в ее теле.
        // Именно поэтому требуется склонировать указатель на accepted_connections.
        //

        {
            let connections = accepted_connections.clone();
            spawn(move || {
                handle_connection(connection_id, connection, &connections).ok();
                connections.lock().unwrap().remove(&connection_id);
            });
        }
    }
}

fn handle_connection(
    connection_id: Uuid,
    connection: TcpStream,
    all_connections: &AcceptedConnections,
) -> Result<(), Error> {
    let mut reader = BufReader::new(connection);
    let mut message = String::new();
    let mut user_id = None;

    loop {
        message.clear();

        if reader.read_line(&mut message).map_err(|e| Error::IO(e))? == 0 {
            return Ok(());
        }

        let cmd = match Command::new(&message) {
            Ok(value) => value,
            Err(error) => {
                reader
                    .get_mut()
                    .write_all(format!("{error}\n").as_bytes())
                    .ok();
                println!("{error}");
                continue;
            }
        };

        //
        // Команды доступные залогиненым пользователям.
        //

        match cmd {
            Command::Login(cmd) => {
                user_id = Some(cmd.id.clone());
                all_connections
                    .lock()
                    .unwrap()
                    .get_mut(&connection_id)
                    .unwrap()
                    .user_id = Some(cmd.id);
            }
            Command::Message(cmd) => {
                for conn in all_connections.lock().unwrap().values_mut() {
                    if conn.user_id.is_none() {
                        continue;
                    }

                    if conn.user_id == user_id {
                        continue;
                    }

                    conn.connection
                        .write_all(
                            format!("{}: {}", user_id.clone().unwrap(), cmd.message).as_bytes(),
                        )
                        .ok();
                }
            }
            _ => continue,
        }
    }
}
