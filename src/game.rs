use moves::Moves;
use board::Board;
use piece::Piece;

pub struct Game {
    pub moves: Moves,
    pub board: Board,
}

impl Game {
    pub fn new() -> Game {
        Game { moves: Moves::new(),
               board: Board::new() }
    }

    pub fn place(&self, piece: &Piece, x: i32, y: i32) -> Game {
        let (next_board, z) = self.board.insert(piece, x, y);
        let next_moves = self.moves.place(piece, x, y, z);
        Game { moves: next_moves, board: next_board }
    }

}
