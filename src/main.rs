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
use piece::UNIQUE_PIECE_COUNT;

fn main() {
    let mut ordered : Vec<usize> = (0..3_usize.pow(UNIQUE_PIECE_COUNT as u32)).collect();
    ordered.sort_by(|a, b| Bag::from_usize(*a).len().cmp(&Bag::from_usize(*b).len()));

    let results = RwLock::new(Results::new());

    for i in ordered {
        println!("======================================================================");
        println!("TESTING {}", i);
        let mut worker = Worker::new(i, &results);
        worker.run();
    }
    println!("Hello, world");
}
