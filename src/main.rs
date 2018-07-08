use std::collections::HashSet;
use std::sync::RwLock;

mod state;
mod piece;
mod board;

use piece::{Pieces, Id};
use state::{State, PIECE_COUNT};
use board::Board;

struct Worker {
    best_score: i32,
    best_state: State,
    pieces: Pieces,
    seen: HashSet<State>,
}

impl Worker {
    fn new() -> Worker {
        Worker {
            best_score: -1,
            best_state: State::new(),
            pieces: Pieces::new(),
            seen: HashSet::new(),
        }
    }

    fn print_state(&self, state: &State) {
        println!("------------------------------------------------------------");
        for layer in 0..state.layers() + 1 {
            Board::from_state(&state.layer(layer as u8), &self.pieces).print();
            print!("\n");
        }
    }

    fn run(&mut self, state: &State, results: &Results) {
        if self.seen.contains(&state)
        {
            return;
        }
        self.seen.insert(state.clone());

        let score = state.score() as i32;
        if score > self.best_score {
            self.best_score = score;
            self.best_state = state.clone();
            //self.print_state(state);
        }

        let mut todo: Vec<State> = Vec::new();
        let board = Board::from_state(&state, &self.pieces);

        for i in state.unplaced().rev() {
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
            let b = next.unplaced_bitfield();
            debug_assert!(results.scores[b] != -1);

            let layers = next.layers();
            debug_assert!(layers != -1);

            let max_score = next.score() as i32 +
                            (layers + 1) * results.deltas[b];
            if max_score > self.best_score {
                self.run(next, results);
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
}

////////////////////////////////////////////////////////////////////////////////

fn main() {
    let mut todo: Vec<usize> = (0..(1<<PIECE_COUNT)).collect();
    todo.sort_by(|a, b| a.count_ones().cmp(&b.count_ones()));

    let mut results = Results::new();

    let mut done = 0;
    let mut global_best = 0;
    let count = todo.len();
    for t in todo {
        let mut state = State::new();
        for i in 0..PIECE_COUNT {
            if t & (1 << i) == 0 {
                state = state.discard(Id(i));
            } else {
                results.deltas[t] += (i >> 1) as i32;
            }
        }

        let mut worker = Worker::new();
        worker.run(&state, &results);
        results.scores[t] = worker.best_score;

        if worker.best_score > global_best {
            println!("------------------------------------------------------------");
            println!("Got new global best");
            for layer in 0..worker.best_state.layers() + 1 {
                Board::from_state(&worker.best_state.layer(layer as u8),
                                  &worker.pieces).print();
                print!("\n");
                global_best = worker.best_score;
            }
        }

        done += 1;
        println!("{} / {} ({}%)", done, count, 100f32 * done as f32 / count as f32);
    }
}

#[cfg(test)]
mod tests {
    use piece::{Pieces, Piece, Id};
    use state::State;
    use board::Board;

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
        assert_eq!(b.check(&zero, 4, 0), -1);
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
}
