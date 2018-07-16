use arrayvec::ArrayVec;
use std::cmp::Ordering;

use piece::{UNIQUE_PIECE_COUNT, MAX_ROTATIONS};

////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

#[derive(Clone, Debug)]
pub struct State {
    pub pieces: ArrayVec<[Placed; UNIQUE_PIECE_COUNT * 2]>,
}

impl State {
    pub fn new() -> State {
        State { pieces: ArrayVec::new() }
    }
    pub fn insert(&self, p: Placed) -> State {
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
        self.pieces.iter().map(|p| { (p.id / MAX_ROTATIONS) * p.z }).sum()
    }
    pub fn size(&self) -> (i32, i32) {
        (self.pieces.iter().map(|p| p.x + 4).max().unwrap_or(0),
         self.pieces.iter().map(|p| p.y + 4).max().unwrap_or(0))
    }
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
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
}
