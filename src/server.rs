use simple_chat::commands::Command;
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
    //
    // Создаем экземпляр буфера для чтения данных из клиентского соединения.
    // Bufread позволяет нам удобным образом считывать команды из соединения
    // благодаря наличию метода read_line.
    //

    let mut reader = BufReader::new(connection);

    //
    // Строка, куда будет временно записыватся каждая команда, поступающая от клиента.
    // Эта строка нужна для метода read_line.
    //

    let mut message = String::new();

    //
    // Идентификатор (пока что имя) пользователя.
    // При выполнении команды Login имя пользователя записывается сюда.
    // На данном этапе у меня нет значения user_id, поэтому инициализирую переменную значением None.
    //

    let mut user_id = None;

    //
    // В бесконечном цикле читаем команды, поступающие от клиента...
    //

    loop {
        //
        // Очищаю строку message, т.к. метод read_line не делает этого автоматически.
        //

        message.clear();

        //
        // Поток блокируется на вызове .read_line до тех пор, пока reader
        // не прочтет полное сообщение от клиента, которое оканчивается служебным символом \n.
        // Если read_line возвращает Ok, а внутри - 0 прочитанных байт, то это значит, что соединение разорвано.
        // В случае разрыва соединения мы завершаем ф-цию handle_connection.
        // Полученная от клиента команда запишется в message по мутабельной ссылке.
        //

        if reader.read_line(&mut message).map_err(|e| Error::IO(e))? == 0 {
            return Ok(());
        }

        //
        // Передаем ссылку на message в конструктор Command, где происходит
        // парсинг команды. Если парсинг парсинг не удался (неверная команда), то
        // мы получаем мутабельную ссылку на пользовательское соединение из reader
        // и отправляем ошибку клиенту.
        // .ok() после write_all игнорирует возможную ошибку отправки данных в сеть.
        //

        let cmd = match Command::new(&message) {
            Ok(value) => value,
            Err(error) => {
                reader
                    .get_mut()
                    .write_all(format!("{error}\n").as_bytes())
                    .ok();
                continue;
            }
        };

        //
        // Определяем что за команда пришла от клиента и выполняем ее.
        //

        match cmd {
            //
            // Клиент хочет залогиниться, присылает свое имя (в будущем - ID).
            // Мы записываем имя пользователя в переменную user_id.
            // Также мы находим текущее подключение (connection) в мапе всех подключений по его ID,
            // а затем также записываем имя пользователя в экземпляр структуры AcceptedConnection (это значение мапы, а ключ - connection_id);
            //
            Command::Login(cmd) => {
                user_id = Some(cmd.id.clone());
                all_connections
                    .lock()
                    .unwrap()
                    .get_mut(&connection_id)
                    .unwrap()
                    .user_id = Some(cmd.id);
            }
            //
            // Клиент прислал сообщение, которое нужно разослать всем остальным клиентам.
            //
            Command::Message(cmd) => {
                //
                // Если пользователь приславший сообщение не залогинен - ничего не делаем.
                //

                if user_id.is_none() {
                    continue;
                }

                //
                // Мы пробегаемся по всем существующим на данный момент клиентским соединениям.
                // В каждое соединение, кроме текущего (которое мы сейчас обрабатываем в ф-ции handle_connection),
                // мы должны отправить сообщение, только что полученное от пользователя.
                //

                for conn in all_connections.lock().unwrap().values_mut() {
                    //
                    // Если пользователя, на чье соединение мы сейчас смотрим (conn) не залогинен - ничего не делаем.
                    //

                    if conn.user_id.is_none() {
                        continue;
                    }

                    //
                    // Если соединение текущей итерации (conn) - это соединение пользователя, приславшего сообщение - ничего не делаем.
                    // Мы не хотим отправлять его же сообщение ему обратно.
                    //

                    if conn.user_id == user_id {
                        continue;
                    }

                    //
                    // Наконец мы отправляем только что полученно сообщение в некоторое соединение.
                    // Ошибку игнорируем.
                    //

                    conn.connection
                        .write_all(
                            format!("{}: {}", user_id.clone().unwrap(), cmd.message).as_bytes(),
                        )
                        .ok();
                }
            }
            //
            // TODO: обработать остальные команды.
            //
            _ => continue,
        }
    }
}
