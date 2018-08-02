extern crate arrayvec;
extern crate colored;
extern crate rayon;

#[macro_use]
extern crate lazy_static;

mod tables;
mod graph;
mod piece;

use graph::Graph;

fn main() {
    Graph::build();
}
