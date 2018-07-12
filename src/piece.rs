use arrayvec::ArrayVec;

use std::cmp::{max, min};
use std::collections::HashMap;

struct Piece {
    pub pts: Vec<(i32, i32)>
}

impl Piece {
    // Interprets a u16 as a 4x4 bitfield and unpacks it into a Piece
    fn from_u16(p: u16) -> Piece {
        let mut out = Piece { pts: Vec::new() };
        for i in 0..16 {
            if (p & (1 << i)) != 0 {
                out.pts.push((i % 4, i / 4));
            }
        }
        return out;
    }

    // Converts a Piece to a unique packed binary representation
    fn to_u16(&self) -> u16 {
        let mut out = 0;
        for p in self.pts.iter() {
            debug_assert!(p.0 >= 0);
            debug_assert!(p.0 <  4);
            debug_assert!(p.1 >= 0);
            debug_assert!(p.1 <  4);
            out |= 1 << (p.0 + p.1 * 4);
        }
        return out;
    }

    // Rotates a Piece by 90° clockwise
    fn rot(&self) -> Piece {
        Piece {
            pts: self.pts.iter().map(|&(x, y)| (y, -x + 3)).collect()
        }
    }
}

const UNIQUE_PIECE_COUNT: usize = 10;
const MAX_ROTATIONS: usize = 4;
const MAX_EDGE_LENGTH: usize = 4;
const OVERLAP_SIZE: usize = 3 * MAX_EDGE_LENGTH;

static PIECES: [u16; UNIQUE_PIECE_COUNT] = [
0b1110101010101110, // 0
0b1100010001000100, // 1
0b0110011011001110, // 2
0b1110001001101110, // 3
0b0110010011100110, // 4
0b1110100011101110, // 5
0b1100100011101110, // 6
0b1110010011001000, // 7
0b0110011011001100, // 8
0b1110111011001100, // 9
];

enum Overlap {
    None,
    Full,
    Partial(usize),
    Neighbor,
}

struct Boop {
    // The core 10 pieces, as indices, in their 4 possible rotations
    pieces: [ArrayVec<[usize; MAX_ROTATIONS]>; UNIQUE_PIECE_COUNT],

    // Bidirectional mapping from packed bitmaps to indices
    ids: HashMap<u16, usize>,
    bmps: HashMap<usize, u16>,

    table: Vec<ArrayVec<[[Overlap; OVERLAP_SIZE * OVERLAP_SIZE];
                          UNIQUE_PIECE_COUNT * MAX_ROTATIONS]>>,
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use piece::Piece;

    #[test]
    fn construction() {
        for i in 0..65535 {
            let p = Piece::from_u16(i);
            assert_eq!(p.to_u16(), i);
        }
    }

    #[test]
    fn rot() {
        for i in 0..65535 {
            let p = Piece::from_u16(i);
            assert_eq!(p.rot().rot().rot().rot().to_u16(), i);
        }
    }
}

/*
/* 8 */ "
 ##
 ##
## 
## ",
/* 9 */ "
###
###
## 
## "];

#[derive(Debug)]
pub struct Piece {
    pub pts: Vec<(i32, i32)>,
    pub neighbors: Vec<(i32, i32)>,
    pub w: i32,
    pub h: i32,
    pub id: Id,
    pub rot: u8,
}

impl Piece {
    fn new() -> Piece {
        Piece { pts: Vec::new(), neighbors: Vec::new(),
                w: 0, h: 0, id: Id(0xFF), rot: 0 }
    }
    pub fn from_id(id: Id) -> Piece {
        Piece { id: id, .. Piece::from_string(PIECE_STRS[id.0 >> 1]) }
    }
    fn from_string(s: &str) -> Piece {
        let mut out = Piece::new();
        for (y, line) in (&s[1..]).to_string().split('\n').rev().enumerate() {
            for (x, chr) in line.chars().enumerate() {
                if chr == '#' {
                    out.pts.push((x as i32, y as i32));
                    out.w = max(x as i32 + 1, out.w);
                    out.h = max(y as i32 + 1, out.h);
                }
            }
        }

        // Find points that share an edge with set tiles but are not set.
        //
        // This is used to enforce the rule that a piece must share an edge
        // with an existing tile on the same layer.
        for &(x, y) in out.pts.iter() {
            for &(dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter() {
                let x = (x as i32) + dx;
                let y = (y as i32) + dy;
                let mut hit = false;
                for &(x_, y_) in out.pts.iter() {
                    if (x_ as i32) == x && (y_ as i32) == y {
                        hit = true;
                        break;
                    }
                }
                for &(x_, y_) in out.neighbors.iter() {
                    if (x_ as i32) == x && (y_ as i32) == y {
                        hit = true;
                        break;
                    }
                }
                if !hit {
                    out.neighbors.push((x, y));
                }
            }
        }
        out
    }

    fn rot(&self) -> Piece {
        let mut out = Piece {
            w: self.h, h: self.w, id: self.id, rot: self.rot + 1,
            .. Piece::new() };

        let mut xmin = 0;
        let mut ymin = 0;
        for &(x, y) in self.pts.iter() {
            let next = (y, -x);
            xmin = min(xmin, next.0);
            ymin = min(ymin, next.1);
            out.pts.push(next);
        }
        for pt in &mut out.pts {
            pt.0 -= xmin;
            pt.1 -= ymin;
        }
        for &(x, y) in self.neighbors.iter() {
            out.neighbors.push((y - xmin, -x - ymin));
        }
        out
    }

    fn rots(self) -> [Piece; 4]
    {
        let b = self.rot();
        let c = b.rot();
        let d = c.rot();
        [self, b, c, d]
    }
}

////////////////////////////////////////////////////////////////////////////////

// Piece Id (ranges from 0 to 19, since we have two of each piece)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Id(pub usize);

////////////////////////////////////////////////////////////////////////////////

pub struct Pieces {
    data: [[Piece; 4]; PIECE_COUNT],
}

impl Pieces {
    pub fn new() -> Pieces {
        Pieces { data: [
            Piece::from_id(Id(0)).rots(),
            Piece::from_id(Id(1)).rots(),
            Piece::from_id(Id(2)).rots(),
            Piece::from_id(Id(3)).rots(),
            Piece::from_id(Id(4)).rots(),
            Piece::from_id(Id(5)).rots(),
            Piece::from_id(Id(6)).rots(),
            Piece::from_id(Id(7)).rots(),
            Piece::from_id(Id(8)).rots(),
            Piece::from_id(Id(9)).rots(),
            Piece::from_id(Id(10)).rots(),
            Piece::from_id(Id(11)).rots(),
            Piece::from_id(Id(12)).rots(),
            Piece::from_id(Id(13)).rots(),
            Piece::from_id(Id(14)).rots(),
            Piece::from_id(Id(15)).rots(),
            Piece::from_id(Id(16)).rots(),
            Piece::from_id(Id(17)).rots(),
            Piece::from_id(Id(18)).rots(),
            Piece::from_id(Id(19)).rots(),
        ]}
    }

    pub fn at(&self, i: Id, rot: usize) -> &Piece {
        &self.data[i.0][rot]
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use piece::Pieces;

    #[test]
    fn pieces() {
        for piece_rots in Pieces::new().data.iter() {
            for piece in piece_rots.iter() {
                for pt in piece.pts.iter() {
                    assert!(pt.0 >= 0);
                    assert!(pt.1 >= 0);
                }
                assert!(piece.w > 0);
                assert!(piece.h > 0);
            }
        }
    }
}
*/
