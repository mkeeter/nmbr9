use std::collections::HashSet;

mod state;
mod piece;
mod board;

use piece::{Pieces, Id};
use state::State;
use board::Board;

struct Worker {
    best: i32,
    pieces: Pieces,
    seen: HashSet<State>,
}

impl Worker {
    fn new() -> Worker {
        Worker {
            best: -1,
            pieces: Pieces::new(),
            seen: HashSet::new(),
        }
    }

    fn run(&mut self, state: &State) {
        if (state.upper_bound_score() as i32 <= self.best) ||
            self.seen.contains(&state)
        {
            return;
        }
        self.seen.insert(state.clone());

        // Create a list of all the available pieces
        let available = state.z.iter().enumerate()
            .filter(|&(_, z)| { *z == 0xFF })
            .map(|(i, _)| { Id(i) });

        let score = state.score() as i32;
        if score > self.best {
            self.best = score;
            println!("Got score {}", score);

            for layer in 0..state.layers() + 1 {
                Board::from_state(&state.layer(layer as u8), &self.pieces).print();
                println!("\n\n");
            }
        }

        let mut todo: Vec<State> = Vec::new();
        let board = Board::from_state(&state, &self.pieces);
        for i in available {
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
            self.run(next);
        }
    }
}


fn main() {
    let state = State::new();

    let mut worker = Worker::new();
    worker.run(&state);
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
