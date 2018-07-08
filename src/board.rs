use std::cmp::max;

use piece::{Id, Pieces, Piece};
use state::State;

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Debug, Clone)]
struct Cell {
    id: Id,
    z: i32,
}

#[derive(Debug)]
pub struct Board {
    grid: Vec<Cell>,
    pub w: i32,
    pub h: i32,
    pub z: i32,
}

impl Board {
    pub fn from_state(state: &State, pieces: &Pieces) -> Board {
        let (w, h) = state.size(pieces);
        let n = (w * h) as usize;
        let mut out = Board {
            grid: vec![Cell { id: Id(0xFF), z: -1 }; n],
            w: w,
            h: h,
            z: -1,
        };

        for i in state.placed() {
            let x = state.x[i.0] as i32;
            let y = state.y[i.0] as i32;

            for &(px, py) in state.get(i, pieces).pts.iter() {
                debug_assert!(px >= 0);
                debug_assert!(py >= 0);

                let z = state.z(i);
                let j = out.index(px + x, py + y);

                debug_assert!(z != out.grid[j].z);
                if z > out.grid[j].z {
                    out.grid[j].id = i;
                    out.grid[j].z = z;
                }
                out.z = max(z, out.z);
            }
        }
        return out;
    }

    fn at(&self, x: i32, y: i32) -> Cell {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.grid[self.index(x, y)]
        } else {
            Cell { id: Id(0xFF), z: -1 }
        }
    }

    pub fn print(&self) {
        for x in 0..self.w {
            for y in (0..self.h).rev() {
                let c = self.at(x, y);
                if c.z == -1 {
                    print!(" ");
                } else {
                    print!("{}", c.id.0 >> 1);
                }
            }
            print!("\n");
        }
    }

    fn index(&self, x: i32, y: i32) -> usize {
        debug_assert!(x >= 0);
        debug_assert!(y >= 0);
        (y * self.w + x) as usize
    }

    // Checks whether a piece can be placed at the given location
    // Returns the Z value of the to-be-placed piece, or -1
    pub fn check(&self, p: &Piece, x: i32, y: i32) -> i32 {
        // Special-case: if the board is empty, then we can place at 0,0
        if self.w == 0 && self.h == 0 {
            if x == 0 && y == 0 {
                return 0;
            } else {
                return -1;
            }
        }

        #[derive(Eq, PartialEq)]
        enum Over { Zero, One(Id), TwoOrMore };

        // Stores the index of pieces that we're placed above
        let mut over = Over::Zero;
        let mut z: Option<i32> = None;

        // Iterate over every point in the piece, checking its Z level
        // and storing whether it overlaps with at least two other pieces
        for &(px, py) in &p.pts {
            let c = self.at(x + px, y + py);

            // If this Z disagrees with our stored Z value, then
            // we know that the piece can't be placed here.
            match z {
                None => z = Some(c.z),
                Some(z_) => if z_ != c.z { return -1 }
            }

            // Count the number of pieces that we've placed over
            match over {
                Over::Zero => if c.id != Id(0xFF) { over = Over::One(c.id) }
                Over::One(id) => if c.id != id { over = Over::TwoOrMore }
                Over::TwoOrMore => ()
            }
        }
        let z = z.unwrap() + 1;

        // If we're placing this piece off of ground level, it must be
        // positioned over at least two other pieces
        if z > 0 && over != Over::TwoOrMore {
            return -1;
        }

        // Finally, check to see whether we're sharing an edge with any
        // other pieces at the new Z level.
        for &(px, py) in &p.neighbors {
            if self.at(x + px, y + py).z == z {
                return z;
            }
        }
        // Otherwise, the positioning is only valid if this is the first
        // piece on a new layer.
        if z > self.z {
            return z;
        } else {
            return -1;
        }
    }
}

#[cfg(test)]
mod tests {
    use board::Board;
    use piece::{Piece, Id, Pieces};
    use state::State;

    #[test]
    fn from_state() {
        let s = State::new();
        let s = s.place(&Piece::from_id(Id(0)), 0, 0, 0);

        let b = Board::from_state(&s, &Pieces::new());
        assert_eq!(b.w, 3);
        assert_eq!(b.h, 4);
    }
}
