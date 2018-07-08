use std::cmp::{min, max};

use piece::{Piece, Pieces, Id};

const PIECE_COUNT: usize = 20;

// This is a compact representation of placed pieces
//
// The z field is a packed representation, where the lower
// 4 bits represent rotation and the upper 4 represent height
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct State {
    pub x: [u8; PIECE_COUNT],
    pub y: [u8; PIECE_COUNT],
    pub z: [u8; PIECE_COUNT],
}

impl State {
    pub fn new() -> State {
        State {
            x: [0x00; PIECE_COUNT],
            y: [0x00; PIECE_COUNT],
            z: [0xFF; PIECE_COUNT],
        }
    }

    // Gets a placed piece from the array
    pub fn get<'a>(&self, i: Id, pieces: &'a Pieces) -> &'a Piece {
        debug_assert!(self.z[i.0] != 0xFF);
        return pieces.at(i, (self.z[i.0] & 0x0F) as usize);
    }

    // Returns the total number of layers in the state, or -1
    pub fn layers(&self) -> i32 {
        self.z.iter()
            .filter(|&z| { *z != 0xFF })
            .map(|z| *z as i32 >> 4).max().unwrap_or(-1)
    }

    // Returns a filtered state that only includes pieces
    // on a particular layer.  This is useful for debug
    // printing of the game state.
    pub fn layer(&self, a: u8) -> State {
        let mut out = self.clone();
        for z in &mut out.z {
            if (*z >> 4) != a {
                *z = 0xFF;
            }
        }
        return out;
    }

    pub fn size(&self, pieces: &Pieces) -> (i32, i32) {
        let mut w = 0;
        let mut h = 0;
        for i in 0..PIECE_COUNT {
            if self.z[i] != 0xFF {
                let p = self.get(Id(i), pieces);
                w = max(w, self.x[i] as i32 + p.w);
                h = max(h, self.y[i] as i32 + p.h);
            }
        }
        return (w, h);
    }

    pub fn place(&self, piece: &Piece, x: i32, y: i32, z: u8) -> State {
        // Clone the existing state, and assert that this piece hasn't
        // already been placed.
        let mut out = self.clone();
        debug_assert!(out.z[piece.id.0] == 0xFF);

        // Shift the entire game board to put the lowest piece at 0,0
        for px in &mut out.x {
            *px = ((*px as i32) - min(x, 0)) as u8;
        }
        for py in &mut out.y {
            *py = ((*py as i32) - min(y, 0)) as u8;
        }
        out.x[piece.id.0] = max(x, 0) as u8;
        out.y[piece.id.0] = max(y, 0) as u8;
        out.z[piece.id.0] = (z << 4) | piece.rot;
        out
    }

    pub fn score(&self) -> usize {
        let out: usize = self.z.iter().enumerate()
            .filter(|&(_, z)| { *z != 0xFF })
            .map(|(i, z)| (i >> 1) * (*z as usize >> 4))
            .sum();
        out
    }

    // Estimates the highest possible score that can be reached
    // from this game state.
    pub fn upper_bound_score(&self) -> usize {
        let mut max_z: i32 = -1;
        let mut free_tiles = PIECE_COUNT as i32;
        let mut score = self.score();
        // Count up available tiles and the highest Z position
        for z in self.z.iter().filter(|&z| { *z != 0xFF }) {
            max_z = max(*z as i32 >> 4, max_z);
            free_tiles -= 1;
        }
        // Then do a best-case estimate assigning free tiles
        // to an optimistically high level.
        for i in (0..PIECE_COUNT).rev().filter(|i| { self.z[*i] == 0xFF })
        {
            let this_tile_score = i >> 1;
            let this_tile_level = max_z + ((free_tiles + 1) >> 1);
            score += this_tile_score * (this_tile_level as usize);
            free_tiles -= 1;
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use state::State;
    use piece::{Id, Piece, Pieces};

    #[test]
    fn upper_bound_score() {
        let g = State::new();
        {
            let tot: usize = (0..10).map(|x| { x * x }).sum();
            let tot = tot * 2;
            assert_eq!(g.upper_bound_score(), tot);
        }
        // Place the first 0
        let g = g.place(&Piece::from_id(Id(0)), 0, 0, 0);
        let tot = 1 + 2 * (1 + 2) + 3 * (2 + 3) + 4 * (3 + 4) + 5 * (4 + 5)
                 + 6 * (5 + 6) + 7 * (6 + 7) + 8 * (7 + 8) + 9 * (8 + 9) + 10 * 9;
        assert_eq!(g.upper_bound_score(), tot);

        // Place the second 0
        let g = g.place(&Piece::from_id(Id(1)), 0, 0, 0);
        assert_eq!(g.upper_bound_score(), 570);

        // Place the first 1 on level 0
        let g = g.place(&Piece::from_id(Id(2)), 0, 0, 0);
        let tot = 1 * (1 + 2) + 2 * (2 + 3) + 3 * (3 + 4) + 4 * (4 + 5)
                 + 5 * (5 + 6) + 6 * (6 + 7) + 7 * (7 + 8) + 8 * (8 + 9) + 9 * 9;
        assert_eq!(g.upper_bound_score(), tot);
    }

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

