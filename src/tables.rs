use std::collections::{VecDeque, HashMap};

use piece::{UNIQUE_PIECE_COUNT, MAX_ROTATIONS, MAX_EDGE_LENGTH, PIECES};
use piece::{Piece, Overlap};
use state::Placed;

const OVERLAP_SIZE: usize = (2 * MAX_EDGE_LENGTH + 1) as usize;

pub struct Table {
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

pub struct Tables {
    // The core 10 pieces, as indices, in their 4 possible rotations
    pieces: [[usize; MAX_ROTATIONS]; UNIQUE_PIECE_COUNT],

    // Bidirectional mapping from packed bitmaps to indices
    bmps: HashMap<usize, u16>,
    ids: HashMap<u16, usize>,

    tables: Vec<Table>
}

impl Tables {
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

    fn build_tables() -> Tables {
        let mut todo = VecDeque::new();

        let mut out = Tables {
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

    fn check(&self, id: usize, x: i32, y: i32, p: &Placed) -> Overlap {
        self.tables[id].at(x - p.x, y - p.y, p.id / 4, p.id % 4)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use piece::Overlap;

    #[test]
    fn tables() {
        let b = Tables::build_tables();
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