extern crate arrayvec;
extern crate colored;
extern crate rayon;

#[macro_use]
extern crate lazy_static;

use std::sync::RwLock;
use std::time::SystemTime;
use rayon::prelude::*;

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

fn run(combos: &[usize], results: &RwLock<Results>) {
    let _: Vec<bool> = combos.par_iter().map(
        |i| {
            let mut worker = Worker::new(*i, results);
            worker.run();
            true
        }).collect();
}

fn main() {
    let mut ordered : Vec<usize> = (0..3_usize.pow(UNIQUE_PIECE_COUNT as u32)).collect();
    ordered.sort_by(|a, b| Bag::from_usize(*a).len().cmp(&Bag::from_usize(*b).len()));

    let results = RwLock::new(Results::new());
    let start_time = SystemTime::now();

    let mut start = 0;
    for num in 0..(2 * UNIQUE_PIECE_COUNT) {
        let mut end = start;
        while Bag::from_usize(ordered[end]).len() <= num
        {
            end += 1;
        }

        println!("============================================================");
        println!("BEGINNING {}-PIECE COMBINATIONS ({} to do)", num, end - start);
        run(&ordered[start..end], &results);
        println!("FINISHED {}-piece tests in {:?}", num, start_time.elapsed());
        start = end;
    }
}
