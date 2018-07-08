mod moves;
mod piece;
mod board;
mod game;

pub use piece::{Piece, Pieces, Id};
pub use game::Game;

struct Worker {
    best: usize,
    pieces: Pieces,
}

impl Worker {
    fn new() -> Worker {
        Worker { best: 0, pieces: Pieces::new() }
    }

    fn run(&mut self, game: &Game) {

        // Create a list of all the available pieces
        let available = game.moves.z.iter().enumerate()
            .filter(|&(_, z)| { *z == 0xFF })
            .map(|(i, _)| { Id(i) });

        let mut todo: Vec<Game> = Vec::new();
        for i in available {
            for rot in 0..4 {
                let p = self.pieces.at(i, rot);

                for x in (-p.w)..(game.board.w + p.w) {
                    for y in (-p.h)..(game.board.h + p.h) {
                        let z = game.board.check(p, x, y);

                        /*
                        if z >= 0
                        {
                            let next_state = state.place(i, x, y, rot as u8, z as u8);
                            let next_board = board.insert(self.pieces.at(i, rot), x, y);
                            let next_score = next_state.score();
                            todo.push((next_state, next_board, next_score));
                        }
                        */
                    }
                }
            }
        }
        /*

        let mut todo: Vec<(GameState, Board, usize)> = Vec::new();
        for i in available {
            for rot in 0..4 {
                let p = self.pieces.at(i, rot);
                for x in (-p.w)..(board.w + p.w) {
                    for y in (-p.h)..(board.h + p.h) {
                        let z = board.check(p, x, y);
                        if z >= 0
                        {
                            let next_state = state.place(i, x, y, rot as u8, z as u8);
                            let next_board = board.insert(self.pieces.at(i, rot), x, y);
                            let next_score = next_state.score();
                            todo.push((next_state, next_board, next_score));
                        }
                    }
                }
            }
        }
        todo.sort_by(|a, b| { a.2.cmp(&b.2) });
        for (next_state, next_board, score) in todo {
            self.run(&next_state, &next_board);
        }
        */
    }
}


fn main() {
    let game = Game::new();

    let mut worker = Worker::new();
    worker.run(&game);
}
