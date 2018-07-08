use std::cmp::{min, max};

use piece::{Piece, Pieces, Id};

pub const PIECE_COUNT: usize = 20;

// This is a compact representation of placed pieces
//
// The d field is a packed representation
//  [z3, z2, z1, z0, s1, s0, r1, r0]
//   7                           0
// where z is height, s is status, and r is rotation
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct State {
    pub x: [u8; PIECE_COUNT],
    pub y: [u8; PIECE_COUNT],
    d: [u8; PIECE_COUNT],
}

#[derive(PartialEq, Eq)]
enum Status {
    Placed,
    Unplaced,
    Discarded,
}

const STATUS_UNPLACED: u8 = 0x8;
const STATUS_DISCARDED: u8 = 0x4;

impl State {
    pub fn new() -> State {
        State {
            x: [0x00; PIECE_COUNT],
            y: [0x00; PIECE_COUNT],
            d: [STATUS_UNPLACED; PIECE_COUNT],
        }
    }

    // Gets a placed piece from the array
    pub fn get<'a>(&self, i: Id, pieces: &'a Pieces) -> &'a Piece {
        debug_assert!(self.status(i) == Status::Placed);
        return pieces.at(i, self.rot(i))
    }

    fn status(&self, i: Id) -> Status {
        if (self.d[i.0] & STATUS_UNPLACED) != 0 {
            return Status::Unplaced;
        } else if (self.d[i.0] & STATUS_DISCARDED) != 0 {
            return Status::Discarded;
        } else {
            return Status::Placed;
        }
    }

    pub fn rot(&self, i: Id) -> usize {
        debug_assert!(self.status(i) == Status::Placed);
        return (self.d[i.0 as usize] & 0x3) as usize;
    }

    pub fn z(&self, i: Id) -> i32 {
        debug_assert!(self.status(i) == Status::Placed);
        return (self.d[i.0 as usize] >> 4) as i32;
    }

    // Returns the total number of layers in the state, or -1
    pub fn layers(&self) -> i32 {
        self.placed().map(|i| self.z(i)).max().unwrap_or(-1)
    }

    // Returns a filtered state that only includes pieces
    // on a particular layer.  This is useful for debug
    // printing of the game state.
    pub fn layer(&self, a: u8) -> State {
        let mut out = self.clone();
        for d in &mut out.d {
            if (*d >> 4) != a {
                *d = STATUS_UNPLACED;
            }
        }
        return out;
    }

    pub fn size(&self, pieces: &Pieces) -> (i32, i32) {
        let mut w = 0;
        let mut h = 0;
        for i in self.placed() {
            let p = self.get(i, pieces);
            w = max(w, self.x[i.0] as i32 + p.w);
            h = max(h, self.y[i.0] as i32 + p.h);
        }
        return (w, h);
    }

    pub fn place(&self, piece: &Piece, x: i32, y: i32, z: u8) -> State {
        // Clone the existing state, and assert that this piece hasn't
        // already been placed.
        let mut out = self.clone();
        debug_assert!(self.status(piece.id) == Status::Unplaced);

        // Shift the entire game board to put the lowest piece at 0,0
        for px in &mut out.x {
            *px = ((*px as i32) - min(x, 0)) as u8;
        }
        for py in &mut out.y {
            *py = ((*py as i32) - min(y, 0)) as u8;
        }
        out.x[piece.id.0] = max(x, 0) as u8;
        out.y[piece.id.0] = max(y, 0) as u8;
        out.d[piece.id.0] = (z << 4) | piece.rot;
        out
    }

    pub fn score(&self) -> usize {
        self.placed()
            .map(|i| (i.0 >> 1) * (self.z(i) as usize))
            .sum()
    }

    pub fn placed<'a>(&'a self) -> impl DoubleEndedIterator<Item=Id> + 'a {
        (0..PIECE_COUNT)
            .map(|i| Id(i))
            .filter(move |i| self.status(*i) == Status::Placed)
    }

    pub fn unplaced<'a>(&'a self) -> impl DoubleEndedIterator<Item=Id> + 'a {
        (0..PIECE_COUNT)
            .map(|i| Id(i))
            .filter(move |i| self.status(*i) == Status::Unplaced)
    }

    pub fn unplaced_bitfield(&self) -> usize {
        let mut out = 0;
        for i in self.unplaced() {
            out |= 1 << i.0;
        }
        return out;
    }

    pub fn discard(&self, i: Id) -> State {
        debug_assert!(self.status(i) == Status::Unplaced);
        let mut out = self.clone();
        out.d[i.0] = STATUS_DISCARDED;
        out
    }
}

#[cfg(test)]
mod tests {
    use state::State;
    use piece::{Id, Piece, Pieces};

    #[test]
    fn score() {
        let g = State::new();
        assert_eq!(g.score(), 0);

        let g = g.place(&Piece::from_id(Id(2)), 0, 0, 1);
        assert_eq!(g.score(), 1);

        let g = g.place(&Piece::from_id(Id(3)), 0, 0, 2);
        assert_eq!(g.score(), 3);
    }

    #[test]
    fn size() {
        let s = State::new().place(&Piece::from_id(Id(0)), 0, 0, 0);
        assert_eq!(s.size(&Pieces::new()), (3, 4));
    }
}

