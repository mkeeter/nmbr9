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
use bag::Bag;
use worker::Worker;

fn main() {
    let mut ordered : Vec<usize> = (0..1000).collect();
    ordered.sort_by(|a, b| Bag::from_usize(*a).len().cmp(&Bag::from_usize(*b).len()));

    for i in ordered {
        println!("======================================================================");
        println!("TESTING {}", i);
        let results = RwLock::new(Results::new());
        let mut worker = Worker::new(i, &results);
        worker.run();
    }
    println!("Hello, world");
}
