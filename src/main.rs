extern crate arrayvec;

use std::cmp::max;
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::SystemTime;

mod state;
mod piece;
//mod board;

fn main() {
    println!("Hello, world");
}

/*
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

struct Results {
    // For the set of pieces in a particular configuration,
    // what is the highest possible score (if we start with
    // the pieces placed on an empty, flat table)?
    scores: Vec<i32>,

    // For a particular set of pieces, by how much does the
    // score increase when we go up by one level?
    // The score is of the form a*0 + b*1 + c*2 + d*3 + ...
    // so the delta is simply a + b + c + d
    deltas: Vec<i32>,
}

impl Results {
    fn new() -> Results {
        Results {
            scores: vec![-1; 1 << PIECE_COUNT],
            deltas: vec![ 0; 1 << PIECE_COUNT],
        }
    }

    fn max_score(&self, state: &State) -> i32 {
        let b = state.unplaced_bitfield();

        let available_score = self.scores[b];
        debug_assert!(available_score != -1);

        let layers = state.layers();
        debug_assert!(layers != -1);

        let available_delta = self.deltas[b];

        return state.score() as i32 + available_score +
                        (layers + 1) * available_delta;
    }
}

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
