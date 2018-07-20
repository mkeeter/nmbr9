extern crate arrayvec;
extern crate colored;

#[macro_use]
extern crate lazy_static;

use std::cmp::max;
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::SystemTime;

mod state;
mod piece;
mod tables;
mod bag;

use bag::Bag;
use piece::{MAX_EDGE_LENGTH, UNIQUE_PIECE_COUNT};
use state::{State};

////////////////////////////////////////////////////////////////////////////////

struct Results {
    // For a particular set of pieces (represented by a 10-digit ternary value),
    // what is the highest possible score (if we start with the pieces placed
    // on a flat, empty table)?
    scores: Vec<(usize, usize)>,
}

impl Results {
    fn new() -> Results {
        Results {
            scores: vec![(0, 0); 3_usize.pow(UNIQUE_PIECE_COUNT as u32)],
        }
    }

    // Returns an upper bound score for a given state, with a certain number
    // of pieces remaining in the bag to be placed.
    fn upper_score_bound(&self, bag: &Bag, state: &State) -> usize {
        let b = bag.as_usize();

        let (available_score, available_delta) = self.scores[b];

        let layers = state.layers();
        return available_score + (layers + 1) * available_delta;
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Worker<'a> {
    target: usize,
    best_score: usize,
    best_state: State,
    results: &'a RwLock<Results>,
}

impl<'a> Worker<'a> {
    fn new(target: usize, results: &'a RwLock<Results>) -> Worker<'a> {
        Worker {
            target: target,
            best_score: 0,
            best_state: State::new(),
            results: results,
        }
    }

    pub fn run(&mut self) {
        let bag = Bag::from_usize(self.target);
        let delta = bag.score();
        self.run_(bag, State::new());

        let mut writer = self.results.write().unwrap();
        println!("Recording score {}, {} for {}", self.best_score, delta, self.target);
        writer.scores[self.target] = (self.best_score, delta);
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
                //return;
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


////////////////////////////////////////////////////////////////////////////////

fn run(bag: Bag, state: State) {
    if bag.is_empty() {
        if state.score() > 0 {
            println!("Got terminal state with score {}", state.score());
            state.pretty_print();
        }
        return;
    }

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

    for (p, s) in todo {
        run(bag.take(p), s);
    }
}

fn main() {
    for i in 0..6 {
        let results = RwLock::new(Results::new());
        let mut worker = Worker::new(i, &results);
        worker.run();
    }
    println!("Hello, world");
}

/*
struct Worker<'a> {
    best_score: i32,
    best_state: State,
    tables: &'a Tables,
    results: &'a RwLock<Results>,
    seen: HashSet<State>,
}

use piece::{Pieces, Id};
use state::{State, PIECE_COUNT};
use board::Board;

struct Worker<'a> {
    best_score: i32,
    best_state: State,
    pieces: &'a Pieces,
    results: &'a RwLock<Results>,
    seen: HashSet<State>,
}

impl<'a> Worker<'a> {
    fn new(pieces: &'a Pieces, results: &'a RwLock<Results>) -> Worker<'a> {
        Worker {
            best_score: -1,
            best_state: State::new(),
            pieces: pieces,
            results: results,
            seen: HashSet::new(),
        }
    }

    fn run(&mut self, state: &State) {
        if self.seen.contains(&state)
        {
            return;
        }
        self.seen.insert(state.clone());

        // This is the largest possible score from the current state
        //let my_max_score = self.results.read().unwrap().max_score(state);

        let score = state.score() as i32;
        if score > self.best_score {
            self.best_score = score;
            self.best_state = state.clone();
        }

        let mut todo: Vec<State> = Vec::new();
        let board = Board::from_state(&state, &self.pieces);

        for i in state.available().rev() {
            for rot in 0..4 {
                let p = self.pieces.at(i, rot);

                for x in (-p.w)..(board.w + p.w) {
                    for y in (-p.h)..(board.h + p.h) {
                        let z = board.check(p, x, y);
                        if z != -1
                        {
                            todo.push(state.place(p, x, y, z as u8));
                        }
                    }
                }
            }
        }

        for next in todo.iter() {
            /*
            if my_max_score == self.best_score {
                return;
            }
            */

            let next_max_score = self.results.read().unwrap().max_score(next);
            if next_max_score > self.best_score {
                self.run(next);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////////////

// A swapper produces every bitfield that shares the same number of placed
// tiles as the input bitfield, since there are two copies of each tile.
struct Swapper(usize);

impl Iterator for Swapper {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let mut carry = true;
        let mut out = 0;
        for i in 0..(PIECE_COUNT >> 1) {
            // Mask off a two-bit section
            let mut b = (self.0 >> (2 * i)) & 0x3;

            // Do the logic for a half-adder
            if carry {
                if b == 0x1 {
                    b = 0x2;
                    carry = false;
                } else if b == 0x2 {
                    b = 0x1;
                    carry = true;
                }
            }
            out |= b << (2 * i);
        }
        self.0 = out;
        if carry { None } else { Some(out) }
    }
}

////////////////////////////////////////////////////////////////////////////////

fn main() {
    let pieces = Pieces::new();

    let mut todo: Vec<usize> = (0..(1<<PIECE_COUNT)).collect();
    todo.sort_by(|a, b| a.count_ones().cmp(&b.count_ones()));

    let results = RwLock::new(Results::new());
    let mut global_best = 0;

    {   // Preload the deltas array, since we can do that quickly
        let mut writer = results.write().unwrap();
        for t in todo.iter() {
            for i in 0..PIECE_COUNT {
                if *t & (1 << i) != 0 {
                    writer.deltas[*t] += (i >> 1) as i32;
                }
            }
        }
    }

    let count = todo.len();
    let mut max_bits = 0;
    let mut start_time = SystemTime::now();

    for (done, t) in todo.iter().enumerate() {
        // We spread symmetric results across every possible
        // bitfield, so this one could be finished before we get
        // to it.
        let percent_done = 100f32 * done as f32 / count as f32;
        if results.read().unwrap().scores[*t] != -1 {
            continue;
        }

        let this_bits = t.count_ones();
        if this_bits > max_bits {
            println!("\n============================================================");
            println!("Completed all {}-bit patterns in {:?}",
                     max_bits, start_time.elapsed().unwrap());
            println!("============================================================");
            max_bits = this_bits;
            start_time = SystemTime::now();
        }

        let mut state = State::new();
        for i in 0..PIECE_COUNT {
            if t & (1 << i) == 0 {
                state = state.discard(Id(i));
            }
        }

        let mut best_subscore = 0;
        for u in todo.iter() {
            if u.count_ones() >= this_bits {
                break;
            }
            else if *u & *t == *u {
                let this_subscore = results.read().unwrap().scores[*u];
                best_subscore = max(best_subscore, this_subscore);
            }
        }

        let mut worker = Worker::new(&pieces, &results);
        worker.best_score = best_subscore;

        worker.run(&state);

        {   // Apply these results to every symmetric set of pieces
            let mut writer = results.write().unwrap();
            writer.scores[*t] = worker.best_score;
            for u in Swapper(*t) {
                writer.scores[u] = worker.best_score;
            }
        }

        if worker.best_score > global_best {
            println!("\n------------------------------------------------------------");
            println!("Got new global best: {}", worker.best_score);
            for layer in 0..worker.best_state.layers() + 1 {
                Board::from_state(&worker.best_state.layer(layer as u8),
                                  &worker.pieces).print();
                print!("\n");
                global_best = worker.best_score;
            }
            println!("------------------------------------------------------------");
        }

        print!("\r{} / {} ({}%) [{}, {}, {}]                       ",
               done, count, percent_done, t, worker.best_score, best_subscore);
    }
}

#[cfg(test)]
mod tests {
    use piece::{Pieces, Piece, Id};
    use state::State;
    use board::Board;
    use Swapper;

    #[test]
    fn gameplay() {
        let pieces = Pieces::new();
        let s = State::new();
        assert_eq!(s.score(), 0);

        let b = Board::from_state(&s, &pieces);
        let zero = Piece::from_id(Id(0));
        assert_eq!(b.check(&zero, 0, 0), 0);
        assert_eq!(b.check(&zero, 1, 0), -1);

        let s = s.place(&zero, 0, 0, 0);
        assert_eq!(s.score(), 0);
        let b = Board::from_state(&s, &pieces);

        let zero = Piece::from_id(Id(1));
        assert_eq!(b.check(&zero, 0, 0), -1);
        assert_eq!(b.check(&zero, 2, 0), -1);
        assert_eq!(b.check(&zero, 3, 0), 0);

        let s = s.place(&zero, 3, 0, 0);
        assert_eq!(s.score(), 0);
        let b = Board::from_state(&s, &pieces);

        let one = Piece::from_id(Id(2));
        assert_eq!(b.check(&one, 0, 0), -1);
        assert_eq!(b.check(&one, 1, 0), -1);
        assert_eq!(b.check(&one, 3, 0), -1);
        assert_eq!(b.check(&one, 2, 0), 1);

        let s = s.place(&one, 2, 0, 1);
        assert_eq!(s.score(), 1);
    }

    #[test]
    fn swapper() {
        let mut s = Swapper(1);
        assert_eq!(s.next(), Some(2));
        assert_eq!(s.next(), None);

        let mut s = Swapper(2);
        assert_eq!(s.next(), None);

        let mut s = Swapper(3);
        assert_eq!(s.next(), None);

        let mut s = Swapper(5);
        assert_eq!(s.next(), Some(6));
        assert_eq!(s.next(), Some(9));
        assert_eq!(s.next(), Some(10));
        assert_eq!(s.next(), None);
    }
}
*/
