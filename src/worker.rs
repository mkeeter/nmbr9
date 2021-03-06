use std::collections::{HashSet, BTreeMap};
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
    seen: HashSet<State>,
}

impl<'a> Worker<'a> {
    pub fn new(target: usize, results: &'a RwLock<Results>) -> Worker<'a> {
        Worker {
            target: target,
            best_score: 0,
            best_state: State::new(),
            results: results,
            seen: HashSet::new(),
        }
    }

    pub fn run(&mut self) {
        let bag = Bag::from_usize(self.target);
        self.best_score = self.results.read().unwrap().upper_subset_score(&bag);
        println!("Running with {} pieces in the {:?},\nand initial best score {}", bag.len(), bag, self.best_score);
        self.run_(bag, State::new());

        println!("Got result {}\n", self.best_score);
        let mut writer = self.results.write().unwrap();
        writer.write_score(self.target, self.best_score);
    }

    fn run_(&mut self, bag: Bag, state: State) {
        if bag.is_empty() {
            return;
        }
        if self.seen.contains(&state) {
            return;
        }

        let score = state.score();
        if score > self.best_score {
            println!("Got new best score: {}", state.score());
            state.pretty_print();
            self.best_score = score;
            self.best_state = state.clone();
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
        let mut todo = BTreeMap::new();
        let size = state.size();
        for b in bag.into_iter() {
            for x in -MAX_EDGE_LENGTH..=size.0 + MAX_EDGE_LENGTH {
                for y in -MAX_EDGE_LENGTH..=size.1 + MAX_EDGE_LENGTH {
                    if let Some(s) = state.try_place(b, x, y) {
                        let (w, h) = s.size();
                        let k = (-(s.score() as i32), w + h);
                        if !todo.contains_key(&k) {
                            todo.insert(k, Vec::new());
                        }
                        todo.get_mut(&k).unwrap().push((b, s));
                    }
                }
            }
        }

        self.seen.insert(state);

        // Then, recurse and continue running with the placements
        for (_, vec) in todo {
            for (p, s) in vec {
                self.run_(bag.take(p), s);
            }
        }
    }
}
