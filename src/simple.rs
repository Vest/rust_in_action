#[macro_use]
extern crate crossbeam;

use crossbeam::channel::unbounded;
use std::thread;

fn main() {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let _ = tx.send(42);
    });

    select! {
        recv(rx) -> msg => println!("{:?}", msg),
    }
}
