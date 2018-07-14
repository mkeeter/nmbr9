use std::collections::{VecDeque, HashMap};

////////////////////////////////////////////////////////////////////////////////

pub const UNIQUE_PIECE_COUNT: usize = 10;
pub const MAX_ROTATIONS: usize = 4;
pub const MAX_EDGE_LENGTH: i32 = 4;
const OVERLAP_SIZE: usize = (2 * MAX_EDGE_LENGTH + 1) as usize;

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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Piece {
    pts: Vec<(i32, i32)>,
    bmp: u16,
}

impl Piece {
    // Interprets a u16 as a 4x4 bitfield and unpacks it into a Piece
    fn from_u16(p: u16) -> Piece {
        let mut out = Piece { pts: Vec::new(), bmp: p };
        for i in 0..16 {
            if (p & (1 << i)) != 0 {
                out.pts.push((3 - (i % 4), i / 4));
            }
        }
        return out;
    }

    fn to_u16(&self) -> u16 { self.bmp }

    fn from_pts(pts: Vec<(i32, i32)>) -> Piece {
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
    fn rot(&self) -> Piece {
        Piece::from_pts(self.pts.iter().map(|&(x, y)| (y, -x + 3)).collect())
    }

    // Checks for overlap with a second piece offset by some distance
    fn check(&self, other: &Piece, dx: i32, dy: i32) -> RawOverlap {
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
            return RawOverlap::Full;
        } else if out != 0 {
            debug_assert!(!none_over);
            return RawOverlap::Partial(out);
        } else if has_neighbor {
            return RawOverlap::Neighbor;
        } else {
            debug_assert!(none_over);
            return RawOverlap::None;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum RawOverlap {
    None,
    Full,
    Partial(u16),
    Neighbor,
}

impl RawOverlap {
    fn to_overlap(&self, ids: &HashMap<u16, usize>) -> Overlap {
        match self {
            RawOverlap::None => Overlap::None,
            RawOverlap::Full => Overlap::Full,
            RawOverlap::Partial(i) => Overlap::Partial(*ids.get(&i).unwrap()),
            RawOverlap::Neighbor => Overlap::Neighbor,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Overlap {
    None,
    Full,
    Partial(usize),
    Neighbor,
}

////////////////////////////////////////////////////////////////////////////////

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
                            let result = p.check(&t, x, y);
                            if let RawOverlap::Partial(p) = result {
                                if out.store(p).1 {
                                    todo.push_back(p);
                                }
                            }

                            // Tag the result with an index for the overlap,
                            // rather than the raw bitmap
                            let result = result.to_overlap(&out.ids);

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
    use piece::{Piece, Overlap, Boop, RawOverlap, PIECES};

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
                   RawOverlap::Partial(0b1100000000000100));
        assert_eq!(zero.check(&one, 1, 0),
                   RawOverlap::Full);
        assert_eq!(zero.check(&one, -1, 0),
                   RawOverlap::Partial(0b0100010001000100));
        assert_eq!(zero.check(&one, -1, -1),
            RawOverlap::Partial(0b0100010001000000));
        assert_eq!(zero.check(&one, -1, 1),
            RawOverlap::Partial(0b0000010001000100));
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
