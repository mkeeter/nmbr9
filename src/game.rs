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

#[cfg(test)]
mod tests {
    use game::Game;
    use piece::{Id, Piece};

    #[test]
    fn gameplay() {
        let g = Game::new();
        assert_eq!(g.moves.score(), 0);

        let zero = Piece::from_id(Id(0));
        assert!(g.board.check(&zero, 0, 0));
        assert!(!g.board.check(&zero, 1, 0));

        let g = g.place(&zero, 0, 0);
        assert_eq!(g.moves.score(), 0);

        let zero = Piece::from_id(Id(1));
        assert!(!g.board.check(&zero, 0, 0));
        assert!(g.board.check(&zero, 3, 0));
        assert!(!g.board.check(&zero, 4, 0));

        let g = g.place(&zero, 3, 0);
        assert_eq!(g.moves.score(), 0);

        let one = Piece::from_id(Id(2));
        assert!(!g.board.check(&one, 0, 0));
        assert!(!g.board.check(&one, 1, 0));
        assert!(!g.board.check(&one, 3, 0));
        assert!(g.board.check(&one, 2, 0));

        let g = g.place(&one, 2, 0);
        assert_eq!(g.moves.score(), 1);
    }
}
