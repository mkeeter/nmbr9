pub const UNIQUE_PIECE_COUNT: usize = 10;
pub const MAX_ROTATIONS: usize = 4;
pub const MAX_EDGE_LENGTH: i32 = 4;

pub const PIECES: [u16; UNIQUE_PIECE_COUNT] = [
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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Piece {
    pub pts: Vec<(i32, i32)>,
    pub bmp: u16,
}

impl Piece {
    // Interprets a u16 as a 4x4 bitfield and unpacks it into a Piece
    pub fn from_u16(p: u16) -> Piece {
        let mut out = Piece { pts: Vec::new(), bmp: p };
        for i in 0..16 {
            if (p & (1 << i)) != 0 {
                out.pts.push((3 - (i % 4), i / 4));
            }
        }
        return out;
    }

    pub fn to_u16(&self) -> u16 { self.bmp }

    pub fn from_pts(pts: Vec<(i32, i32)>) -> Piece {
        let mut bmp = 0;
        for p in pts.iter() {
            debug_assert!(p.0 >= 0);
            debug_assert!(p.0 <  4);
            debug_assert!(p.1 >= 0);
            debug_assert!(p.1 <  4);
            bmp |= 1 << ((3 - p.0) + p.1 * 4);
        }
        Piece { pts: pts, bmp: bmp }
    }

    fn at(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= 4 || y >= 4 {
            false
        } else {
            (self.bmp & (1 << ((3 - x) + y * 4))) != 0
        }
    }

    // Rotates a Piece by 90° clockwise
    pub fn rot(&self) -> Piece {
        Piece::from_pts(self.pts.iter().map(|&(x, y)| (y, -x + 3)).collect())
    }

    // Checks for overlap with a second piece offset by some distance
    pub fn check(&self, other: &Piece, dx: i32, dy: i32) -> Overlap {
        let mut all_over = true;
        let mut none_over = true;
        let mut has_neighbor = false;
        let mut out: u16 = 0;

        for (x, y) in other.pts.iter() {
            if self.at(x + dx, y + dy) {
                out |= 1 << ((3 - x) + y * 4);
                none_over = false;
            } else {
                all_over = false;
            }

            for &(nx, ny) in [(0, 1), (0, -1), (1, 0), (-1, 0)].iter()
            {
                has_neighbor |= self.at(x + dx + nx, y + dy + ny);
            }
        }

        if all_over {
            debug_assert!(!none_over);
            debug_assert!(out == other.to_u16());
            return Overlap::Full;
        } else if out != 0 {
            debug_assert!(!none_over);
            return Overlap::_Partial(out);
        } else if has_neighbor {
            return Overlap::Neighbor;
        } else {
            debug_assert!(none_over);
            return Overlap::None;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Overlap {
    None,
    Full,
    _Partial(u16),  // Overlap result encoded as bitfield
    Partial(usize), // Overlap result encoded as index
    Neighbor,
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use piece::{Piece, Overlap, PIECES};

    #[test]
    fn construction() {
        for i in 0..65535 {
            let p = Piece::from_u16(i);
            assert_eq!(Piece::from_pts(p.pts).to_u16(), i);
        }
    }

    #[test]
    fn rot() {
        for i in 0..65535 {
            let p = Piece::from_u16(i);
            assert_eq!(p.rot().rot().rot().rot().to_u16(), i);
        }
    }

    #[test]
    fn check() {
        let zero = Piece::from_u16(PIECES[0]);
        let one = Piece::from_u16(PIECES[1]);
        assert_eq!(zero.check(&one, 0, 0),
                   Overlap::_Partial(0b1100000000000100));
        assert_eq!(zero.check(&one, 1, 0),
                   Overlap::Full);
        assert_eq!(zero.check(&one, -1, 0),
                   Overlap::_Partial(0b0100010001000100));
        assert_eq!(zero.check(&one, -1, -1),
            Overlap::_Partial(0b0100010001000000));
        assert_eq!(zero.check(&one, -1, 1),
            Overlap::_Partial(0b0000010001000100));
    }
}
