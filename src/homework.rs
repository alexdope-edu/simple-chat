use std::sync::Mutex;
use std::thread::{sleep, spawn};
use std::{sync::Arc, time::Duration};

fn main() {
    let simbols = Arc::new(Mutex::new(String::new()));

    let mut treads = Vec::new();

    for number in 0..100 {
        let simbols = simbols.clone();

        let (ch, dur) = if number % 2 != 0 { ('a', 1) } else { ('b', 2) };

        treads.push(spawn(move || treads_behaviour(ch, dur, simbols.clone())));
    }
    for handler in treads {
        handler.join().unwrap()
    }
}

fn treads_behaviour(char: char, sleep_duration_sec: u64, simbol: Arc<Mutex<String>>) {
    loop {
        sleep(Duration::from_secs(sleep_duration_sec));

        let mut simbols_guard = simbol.lock().unwrap();

        if simbols_guard.len() == 100 {
            break;
        }
        simbols_guard.push(char);
    }
}

// 1. Создать строку, которую можно безопасно расшарить между несколькими потоками.
// 2. Создать дополнительный поток №1, который каждую секунду будет дописывать в строку символ 'a'.
// 3. Создать дополнительный поток №2, который каждые 2 секунды будет дописывать в строку символ 'б'.
// 4. Каждый из потоков должен проверять, что в строке не более 100 символов. Иначе - поток должен завершиться.

// thread::sleep(Duration::from_secs(1));

// Длину строки можно проще проверить
// Нужны паузы и потоки
// Мьютекс нужно лочить один раз за итерацию loop
