extern crate arrayvec;
extern crate colored;

#[macro_use]
extern crate lazy_static;

use std::sync::RwLock;
use std::time::SystemTime;

mod bag;
mod state;
mod piece;
mod tables;
mod results;
mod worker;

use results::Results;
use worker::Worker;

fn main() {
    for i in 0..6 {
        let results = RwLock::new(Results::new());
        let mut worker = Worker::new(i, &results);
        worker.run();
    }
    println!("Hello, world");
}
