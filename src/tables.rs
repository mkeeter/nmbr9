use std::collections::{VecDeque, HashMap};

use piece::{UNIQUE_PIECE_COUNT, MAX_ROTATIONS, MAX_EDGE_LENGTH, PIECES};
use piece::{Piece, Overlap};

const OVERLAP_SIZE: usize = (2 * MAX_EDGE_LENGTH + 1) as usize;

struct Table {
    data: [Overlap; OVERLAP_SIZE * OVERLAP_SIZE *
                    MAX_ROTATIONS * UNIQUE_PIECE_COUNT],
}

impl Table {
    fn new() -> Table {
        Table { data: [Overlap::None; OVERLAP_SIZE * OVERLAP_SIZE *
                                      MAX_ROTATIONS * UNIQUE_PIECE_COUNT] }
    }

    fn at(&self, x: i32, y: i32, rot: usize, piece: usize) -> Overlap {
        self.data[Table::index(x, y, rot, piece)]
    }

    fn store(&mut self, x: i32, y: i32, rot: usize, piece: usize, d: Overlap) {
        self.data[Table::index(x, y, rot, piece)] = d;
    }

    fn index(x: i32, y: i32, rot: usize, piece: usize) -> usize {
        debug_assert!(piece < UNIQUE_PIECE_COUNT);
        debug_assert!(rot < MAX_ROTATIONS);
        debug_assert!(x <= MAX_EDGE_LENGTH);
        debug_assert!(x >= -MAX_EDGE_LENGTH);
        debug_assert!(y <= MAX_EDGE_LENGTH);
        debug_assert!(y >= -MAX_EDGE_LENGTH);

        let x = (x + MAX_EDGE_LENGTH) as usize;
        let y = (y + MAX_EDGE_LENGTH) as usize;

        x + OVERLAP_SIZE *
            (y + OVERLAP_SIZE *
                (rot + MAX_ROTATIONS * piece))
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Boop {
    // The core 10 pieces, as indices, in their 4 possible rotations
    pieces: [[usize; MAX_ROTATIONS]; UNIQUE_PIECE_COUNT],

    // Bidirectional mapping from packed bitmaps to indices
    bmps: HashMap<usize, u16>,
    ids: HashMap<u16, usize>,

    tables: Vec<Table>
}

impl Boop {
    fn store(&mut self, bmp: u16) -> (usize, bool) {
        match self.ids.get(&bmp) {
            None => {
                let id = self.ids.len();
                self.ids.insert(bmp, id);
                self.bmps.insert(id, bmp);
                return (id, true);
            },
            Some(&id) => return (id, false),
        }
    }

    fn last_table<'a>(&'a mut self) -> &'a mut Table {
        self.tables.last_mut().unwrap()
    }

    fn build_tables() -> Boop {
        let mut todo = VecDeque::new();

        let mut out = Boop {
            pieces: [[0; MAX_ROTATIONS]; UNIQUE_PIECE_COUNT],
            bmps: HashMap::new(),
            ids: HashMap::new(),
            tables: Vec::new(),
        };

        // Construct the 40 original pieces (10 pieces * 4 rotations)
        for i in 0..UNIQUE_PIECE_COUNT {
            let mut p = Piece::from_u16(PIECES[i]);
            for r in 0..MAX_ROTATIONS {
                let b = p.to_u16();
                out.pieces[i][r] = out.store(b).0;
                todo.push_back(b);
                p = p.rot();
            }
        }
        debug_assert!(todo.len() == MAX_ROTATIONS * UNIQUE_PIECE_COUNT);

        // Figure out every pieces that we could put onto one of the original
        // pieces.  In some cases, this produces a new sub-piece, which we add
        // to the queue to be checked in turn.
        while let Some(t) = todo.pop_front() {
            out.tables.push(Table::new());
            println!("Testing {:16b} ({} total)", t, out.tables.len());

            let t = Piece::from_u16(t);

            for i in 0..UNIQUE_PIECE_COUNT {
                let mut p = Piece::from_u16(PIECES[i]);
                for r in 0..MAX_ROTATIONS {
                    for x in -MAX_EDGE_LENGTH..=MAX_EDGE_LENGTH {
                        for y in -MAX_EDGE_LENGTH..=MAX_EDGE_LENGTH {
                            let mut result = p.check(&t, x, y);
                            if let Overlap::_Partial(p) = result {
                                if out.store(p).1 {
                                    todo.push_back(p);
                                }
                                // Tag the result with an index for the overlap,
                                // rather than the raw bitmap
                                result = Overlap::Partial(
                                    *out.ids.get(&p).unwrap());
                            }

                            // Then, store it in the table
                            out.last_table().store(x, y, r, i, result);
                        }
                    }
                    p = p.rot();
                }
            }
        }
        return out;
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use piece::{Piece, Overlap, PIECES};
    use tables::{Boop};

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

    #[test]
    fn boop() {
        let b = Boop::build_tables();
        assert_eq!(b.tables[0].at(0, 0, 0, 0), Overlap::Full);
        assert_eq!(b.tables[0].at(3, 0, 0, 0), Overlap::Neighbor);
        assert_eq!(b.tables[0].at(4, 0, 0, 0), Overlap::None);
        assert_eq!(b.tables[0].at(-3, 0, 0, 0), Overlap::Neighbor);
        assert_eq!(b.tables[0].at(-4, 0, 0, 0), Overlap::None);
        assert_eq!(b.tables[0].at(0, 4, 0, 0), Overlap::Neighbor);
        assert_eq!(b.tables[0].at(0, -4, 0, 0), Overlap::Neighbor);
        assert_eq!(b.tables[0].at(0, -3, 0, 0),
            Overlap::Partial(*b.ids.get(&0b1110000000000000).unwrap()));

        // Overlap a 1 onto a 0 and see that we get the correct pattern out
        assert_eq!(b.tables[4].at(0, 0, 0, 0),
            Overlap::Partial(*b.ids.get(&0b1100000000000100).unwrap()));
        assert_eq!(b.tables[4].at(1, 0, 0, 0), Overlap::Full);
        assert_eq!(b.tables[4].at(-1, 0, 0, 0),
            Overlap::Partial(*b.ids.get(&0b0100010001000100).unwrap()));
        assert_eq!(b.tables[4].at(-1, -1, 0, 0),
            Overlap::Partial(*b.ids.get(&0b0100010001000000).unwrap()));
        assert_eq!(b.tables[4].at(-1, 1, 0, 0),
            Overlap::Partial(*b.ids.get(&0b0000010001000100).unwrap()));
    }
}

////////////////////////////////////////////////////////////////////////////////

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

