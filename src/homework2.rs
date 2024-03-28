use std::{sync::mpsc::channel, thread::spawn};

fn main() {
    let (tx, rx) = channel();

    let handle = spawn(move || {
        let mut numbers = vec![];

        while let Ok(number) = rx.recv() {
            numbers.push(number)
        }
        numbers
    });

    {
        let tx = tx.clone();
        spawn(move || {
            for number in 0..100 {
                tx.send(number).unwrap();
            }
        });
    }

    spawn(move || {
        for number in 100..=200 {
            tx.send(number).unwrap();
        }
    });

    let mut result = handle.join().unwrap();
    result.sort();
    dbg!(result);
}
