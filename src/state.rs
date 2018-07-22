use arrayvec::ArrayVec;
use std::cmp::Ordering;

use colored::*;

use piece::{UNIQUE_PIECE_COUNT, MAX_ROTATIONS, PIECES, PIECE_COLORS, Overlap, Piece};
use tables::{OVERLAP_TABLES};

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Placed {
    id: usize,
    pub x: i32,
    pub y: i32,
    pub z: usize,
}

impl Placed {
    pub fn new(id: usize, x: i32, y: i32, z: usize) -> Placed {
        Placed { id: id, x: x, y: y, z: z}
    }
    pub fn rot(&self) -> usize {
        debug_assert!(self.id < UNIQUE_PIECE_COUNT * MAX_ROTATIONS);
        self.id % MAX_ROTATIONS
    }
    pub fn index(&self) -> usize {
        debug_assert!(self.id < UNIQUE_PIECE_COUNT * MAX_ROTATIONS);
        self.id / MAX_ROTATIONS
    }
}

impl Ord for Placed {
    fn cmp(&self, other: &Placed) -> Ordering {
        if self.z != other.z {
            return other.z.cmp(&self.z);
        } else {
            return (self.id, self.x, self.y).cmp(&(other.id, other.x, other.y));
        }
    }
}

impl PartialOrd for Placed {
    fn partial_cmp(&self, other: &Placed) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub pieces: ArrayVec<[Placed; UNIQUE_PIECE_COUNT * 2]>,
}

impl State {
    pub fn new() -> State {
        State { pieces: ArrayVec::new() }
    }

    // Inserts a new piece, maintaining sorted order
    fn insert(&self, p: Placed) -> State {
        let mut out = self.clone();
        out.pieces.push(p);
        out.pieces.sort_unstable();

        let xmin = out.pieces.iter().map(|p| p.x).min().unwrap();
        let ymin = out.pieces.iter().map(|p| p.y).min().unwrap();
        for p in &mut out.pieces {
            p.x -= xmin;
            p.y -= ymin;
        }

        return out;
    }

    pub fn score(&self) -> usize {
        self.pieces.iter().map(|p| p.index() * p.z).sum()
    }

    pub fn size(&self) -> (i32, i32) {
        (self.pieces.iter().map(|p| p.x + 4).max().unwrap_or(0),
         self.pieces.iter().map(|p| p.y + 4).max().unwrap_or(0))
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub fn layers(&self) -> usize {
        self.pieces.first().map(|p| p.z).unwrap_or(0)
    }

    // Attempts to place a piece at the given position
    pub fn try_place(&self, piece: usize, x: i32, y: i32) -> Option<State> {
        // We only allow the first piece to be placed at the origin,
        // and with zero rotation, to reduce degrees of freedom
        if self.is_empty() {
            if x == 0 && y == 0 {
                let p = Placed::new(piece, x, y, 0);
                if p.rot() == 0 {
                    return Some(self.insert(p));
                }
            }
            return None;
        }

        // Here's the Z layer that we start on!
        let mut current_z = self.pieces.first().map(|p| p.z).unwrap_or(0);

        // Have we seen a neighboring piece on this particular layer?
        let mut got_neighbor_this_layer = false;

        // Did we see a neighboring piece on the previous layer?
        // Pieces being placed above the top layer don't need a neighbor,
        // so we initialize this to true.
        let mut got_neighbor_prev_layer = true;

        // The piece mutates as parts of it are placed over other pieces
        let mut remaining_piece = piece;

        for p in self.pieces.iter() {
            if p.z != current_z {
                // If some of the piece ended up over pieces on this layer,
                // then it will be unsupported, so we must return false.
                if remaining_piece != piece {
                    return None;
                }

                current_z = p.z;
                got_neighbor_prev_layer = got_neighbor_this_layer;
                got_neighbor_this_layer = false;
                remaining_piece = piece;
            }

            let r = OVERLAP_TABLES.at(remaining_piece).check(x, y, &p);
            match r {
                Overlap::_Partial(_) => panic!("Uncleaned index"),
                Overlap::None => (),
                Overlap::Neighbor => got_neighbor_this_layer = true,
                Overlap::Partial(t) => remaining_piece = t,
                Overlap::Full =>
                    if (remaining_piece != piece) && (got_neighbor_prev_layer) {
                        return Some(self.insert(
                                Placed::new(piece, x, y, p.z + 1)));
                    } else {
                        return None;
                    }
            }
        }
        if got_neighbor_this_layer && remaining_piece == piece {
            debug_assert!(current_z == 0);
            return Some(self.insert(Placed::new(piece, x, y, 0)));
        } else {
            return None;
        }
    }

    pub fn pretty_print(&self) {
        let (w, h) = self.size();

        for z in 0..self.pieces.first().map(|p| p.z + 1).unwrap_or(0) {
            let mut v = vec![-1; (w * h) as usize];

            println!("Layer {}:\n", z);
            for i in self.pieces.iter().filter(|&p| p.z == z) {
                let p = Piece::from_u16(PIECES[i.index()]).rotn(i.rot());
                for (px, py) in p.pts {
                    let x = px + i.x;
                    let y = py + i.y;
                    v[((w - x - 1) + y * w) as usize] = i.index() as i32;
                }
            }

            for y in 0..h {
                for x in 0..w {
                    let i = v[(x + y * w) as usize];
                    if i >= 0 {
                        print!("{}", "  ".on_color(PIECE_COLORS[i as usize]))
                    } else {
                        print!("  ");
                    }
                }
                print!("\n");
            }
            for _ in 0..w {
                print!("--");
            }
            print!("\n");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use state::{Placed, State};

    #[test]
    fn score() {
        let state = State::new();
        let state = state.insert(Placed::new(0, 0, 0, 0));
        assert_eq!(state.score(), 0);

        let state = state.insert(Placed::new(4, 0, 0, 1));
        assert_eq!(state.score(), 1);
    }

    #[test]
    fn insert() {
        let state = State::new()
            .insert(Placed::new(0, -1, -2, 1));
        assert_eq!(state.pieces[0], Placed::new(0, 0, 0, 1));
        let state = state.insert(Placed::new(0, -3, -2, 0));
        assert_eq!(state.pieces[0], Placed::new(0, 3, 2, 1));
    }


    #[test]
    fn ordering() {
        let state = State::new()
            .insert(Placed::new(0, 0, 0, 0))
            .insert(Placed::new(4, 0, 0, 1));
        assert_eq!(state.pieces[0], Placed::new(4, 0, 0, 1));
        let state = state.insert(Placed::new(5, 1, 3, 2));
        assert_eq!(state.pieces[0], Placed::new(5, 1, 3, 2));
        let state = state.insert(Placed::new(5, 1, 3, 1));
        assert_eq!(state.pieces[0], Placed::new(5, 1, 3, 2));
    }

    #[test]
    fn size() {
        let state = State::new();
        assert_eq!(state.size(), (0, 0));
        let state = state.insert(Placed::new(5, 0, 0, 1));
        assert_eq!(state.size(), (4, 4));
        let state = state.insert(Placed::new(5, 2, 1, 1));
        assert_eq!(state.size(), (6, 5));
        let state = state.insert(Placed::new(5, -2, 1, 1));
        assert_eq!(state.size(), (8, 5));
    }

    #[test]
    fn try_place() {
        let state = State::new().try_place(0, 0, 0).unwrap();

        assert_eq!(state.try_place(0, 0, 0), None, "perfect overlap");
        assert_eq!(state.try_place(0, 1, 0), None, "imperfect overlap");
        assert_eq!(state.try_place(0, 4, 0), None, "no neighbor");

        state.try_place(0, -3, 0).unwrap(); // No panicking allowed!

        // Place a second 0 next to the first one
        let state = state.try_place(0, 3, 0).unwrap();

        // Now, try placing a 1 in various positions
        assert_eq!(state.try_place(4, 0, 0), None, "imperfect overlap");
        assert_eq!(state.try_place(4, 1, 0), None, "perfect overlap");
        let state = state.try_place(4, 2, 0).unwrap();
        assert_eq!(state.score(), 1);
    }
}
