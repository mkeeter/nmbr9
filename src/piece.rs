use std::cmp::{max, min};

use state::PIECE_COUNT;

static PIECE_STRS: &'static [&'static str] = &[
/* 0 */ "
###
# #
# #
###",
/* 1 */ "
##
 #
 #
 #",
/* 2 */ "
 ##
 ##
## 
###",
/* 3 */ "
###
  #
 ##
###",
/* 4 */ "
 ##
 # 
###
 ##",
/* 5 */ "
###
#  
###
###",
/* 6 */ "
## 
#  
###
###",
/* 7 */ "
###
 # 
## 
#  ",
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
