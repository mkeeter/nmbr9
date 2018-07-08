use std::cmp::{min, max};

use piece::{Piece, Id};

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Debug, Clone)]
struct Cell {
    id: Id,
    z: i8,
}

#[derive(Debug)]
pub struct Board {
    grid: Vec<Cell>,
    pub w: i32,
    pub h: i32,
}

impl Board {
    pub fn new() -> Board {
        Board {
            grid: Vec::new(),
            w: 0,
            h: 0,
        }
    }

    pub fn layers(&self) -> usize {
        (self.grid.iter().map(|&c| { c.z }).max().unwrap_or(-1) + 1)
            as usize
    }

    fn at(&self, x: i32, y: i32) -> Cell {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.grid[self.index(x, y)]
        } else {
            Cell { id: Id(0xFF), z: -1 }
        }
    }

    fn index(&self, x: i32, y: i32) -> usize {
        debug_assert!(x >= 0);
        debug_assert!(y >= 0);
        (y * self.w + x) as usize
    }

    // Checks whether a piece can be placed at the given location
    // Returns the Z value of the to-be-placed piece, or -1
    pub fn check(&self, p: &Piece, x: i32, y: i32) -> i8 {
        // Special-case: if the board is empty, then we can place anywhere
        if self.w == 0 && self.h == 0 {
            return 0;
        }

        #[derive(Eq, PartialEq)]
        enum Over { Zero, One(Id), TwoOrMore };

        // Stores the index of pieces that we're placed above
        let mut over = Over::Zero;
        let mut z: Option<i8> = None;

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
        return -1;
    }

    pub fn insert(&self, p: &Piece, id: Id, x: i32, y: i32) -> Board {
        // Figure out how much we want to shift by
        let xmin = min(x, 0);
        let ymin = min(y, 0);
        let xmax = max(self.w, x + p.w);
        let ymax = max(self.h, y + p.h);

        let mut out = self.expand(
            xmax - xmin, ymax - ymin,
            -xmin, -ymin);

        let x = x - xmin;
        let y = y - ymin;

        for &(px, py) in &p.pts {
            debug_assert!(px >= 0);
            debug_assert!(py >= 0);

            let i = (py + y) * out.w + (px + x);
            debug_assert!(i >= 0);

            let i = i as usize;
            out.grid[i].id = id;
            out.grid[i].z += 1;
        }
        out
    }

    pub fn expand(&self, w: i32, h: i32, dx: i32, dy: i32) -> Board {
        debug_assert!(w >= self.w);
        debug_assert!(h >= self.h);

        let n = (w * h) as usize;
        let mut out = Board {
            grid: Vec::with_capacity(n),
            w: w,
            h: h,
        };

        // Fill the grid with the background cell
        for _ in 0..n {
            out.grid.push(self.at(-1, -1));
        }

        // Then, transplant the old grid onto the new one
        for x in 0..self.w {
            for y in 0..self.h {
                let index_old = self.index(x, y);
                let index_new = out.index(x + dx, y + dy);
                out.grid[index_new] = self.grid[index_old];
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use board::Board;

    #[test]
    fn expand() {
        let b = Board::new();
        let b = b.expand(10, 20, 0, 0);
        assert_eq!(b.w, 10);
        assert_eq!(b.h, 20);

        let b = b.expand(12, 20, 0, 0);
        assert_eq!(b.w, 12);
        assert_eq!(b.h, 20);
    }
}
