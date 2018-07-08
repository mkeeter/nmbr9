use moves::Moves;
use board::Board;

pub struct Game {
    pub moves: Moves,
    pub board: Board,
}

impl Game {
    pub fn new() -> Game {
        Game { moves: Moves::new(),
               board: Board::new() }
    }
}
