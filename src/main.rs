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
        println!("{}", game.moves.score());

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
                        if game.board.check(p, x, y) {
                            todo.push(game.place(p, x, y))
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
    let game = Game::new();

    let mut worker = Worker::new();
    worker.run(&game);
}
