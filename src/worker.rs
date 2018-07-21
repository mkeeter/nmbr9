use std::sync::RwLock;

use results::Results;
use bag::Bag;
use piece::MAX_EDGE_LENGTH;
use state::State;

pub struct Worker<'a> {
    target: usize,
    best_score: usize,
    best_state: State,
    results: &'a RwLock<Results>,
}

impl<'a> Worker<'a> {
    pub fn new(target: usize, results: &'a RwLock<Results>) -> Worker<'a> {
        Worker {
            target: target,
            best_score: 0,
            best_state: State::new(),
            results: results,
        }
    }

    pub fn run(&mut self) {
        let bag = Bag::from_usize(self.target);
        self.best_score = self.results.read().unwrap().upper_subset_score(&bag);
        println!("Running with {} pieces in the bag {:?}, and initial best score {}", bag.len(), bag, self.best_score);
        self.run_(bag, State::new());

        println!("Got result {}", self.best_score);
        let mut writer = self.results.write().unwrap();
        writer.write_score(self.target, self.best_score);
    }

    fn run_(&mut self, bag: Bag, state: State) {
        let score = state.score();
        if score > self.best_score {
            println!("Got new best score: {}", state.score());
            state.pretty_print();
            self.best_score = score;
            self.best_state = state.clone();
        }
        if bag.is_empty() {
            return;
        }

        // Check to see whether we could possibly beat our current
        // best score; otherwise, return immediately.
        if bag.as_usize() != self.target {
            let b = self.results.read().unwrap().upper_score_bound(&bag, &state);
            if b <= self.best_score {
                return;
            }
        }

        // Try placing every piece in the bag onto every possible position
        let mut todo = Vec::new();
        let size = state.size();
        for b in bag.into_iter() {
            for x in -MAX_EDGE_LENGTH..=size.0 + MAX_EDGE_LENGTH {
                for y in -MAX_EDGE_LENGTH..=size.1 + MAX_EDGE_LENGTH {
                    if let Some(s) = state.try_place(b, x, y) {
                        todo.push((b, s));
                    }
                }
            }
        }

        // Then, recurse and continue running with the placements
        for (p, s) in todo {
            self.run_(bag.take(p), s);
        }
    }
}
