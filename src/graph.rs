use std::collections::HashSet;
use std::fmt;
use std::char::from_digit;
use std::collections::HashMap;
use std::time::SystemTime;
use std::cmp::max;

use rayon::prelude::*;

use piece::{PIECE_AREA, UNIQUE_PIECE_COUNT, Overlap};
use tables::OVERLAP_TABLES;

lazy_static! {
    pub static ref PIECES_AREA: Vec<u32> = {
        let mut out = Vec::new();
        for i in 0..((3 as u16).pow(10) + 1) {
            let p = Pieces(i);
            out.push(p.area_())
        }
        return out;
    };
}

/*  Represents a single layer as a ternary value */
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct Pieces(u16);

impl fmt::Debug for Pieces {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pieces(")?;
        for i in (0..10).rev() {
            write!(f, "{}", from_digit(
                    ((self.0 / (3 as u16).pow(i)) % 3) as u32, 3).unwrap())?;
        }
        return write!(f, ")");
    }
}

impl Pieces {
    fn digit(&self, i: usize) -> u16 {
        (self.0 / Pieces::unit(i)) % 3
    }

    fn take(&self, i: usize) -> Pieces {
        debug_assert!(self.digit(i) > 0);
        Pieces(self.0 - Pieces::unit(i))
    }

    fn add(&self, i: usize) -> Pieces {
        debug_assert!(self.digit(i) < 2);
        Pieces(self.0 + Pieces::unit(i))
    }

    fn len(&self) -> usize {
        (0..10).map(|i| self.digit(i) as usize).sum()
    }

    fn unit(i: usize) -> u16 {
        (3 as u16).pow(i as u32)
    }

    fn area_(&self) -> u32 {
        let mut out = 0;
        for i in 0..11 {
            out += PIECE_AREA[i] * (self.digit(i) as u32);
        }
        return out;
    }

    fn area(&self) -> u32 {
        PIECES_AREA[self.0 as usize]
    }

    /*  Returns a set containing the special base piece */
    fn base() -> Pieces {
        Pieces(Pieces::unit(UNIQUE_PIECE_COUNT))
    }

    /*  Returns a set containing all the ordinary pieces */
    fn all() -> Pieces {
        Pieces(Pieces::unit(UNIQUE_PIECE_COUNT) - 1)
    }

    /*  Returns a set containing no pieces */
    fn empty() -> Pieces {
        Pieces(0)
    }

    fn placements(&self) -> Vec<Placement> {
        let mut seen = HashSet::new();

        let mut todo = Vec::new();
        todo.push((Placement::new(), self.clone()));

        // Fully-assembled pieces
        let mut done = Vec::new();

        while todo.len() > 0 {
            let mut next = Vec::new();
            for (placement, remaining) in todo {
                if seen.contains(&placement) {
                    continue;
                }

                if remaining.len() == 0 {
                    done.push(placement);
                } else {
                    // Find which digits still have available pieces
                    for i in (0..10).filter(|i| { remaining.digit(*i) > 0 }) {
                        // Try all possible placements here
                        next.push((placement, remaining.take(i)));
                    }
                }
                seen.insert(placement);
            }
            todo = next;
        }

        return Vec::new();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct Layer {
    placed: Pieces,
    remaining: Pieces,
}

impl Layer {
    pub fn base() -> Layer {
        Layer {
            placed: Pieces::base(),
            remaining: Pieces::all(),
        }
    }

    /*
     *  For a given layer, returns every possible next layer that doesn't
     *  violate overlap or area constraints.
     *
     *  These are found by removing pieces from the 'remaining' set and
     *  moving them into a new, initially-empty 'placed' set.
     */
    pub fn next(&self) -> Vec<Layer> {
        // We need at least two pieces before we can add more pieces on top
        // (this is the overlap constraint)
        let out = Vec::new();
        if self.placed.len() < 2 &&
           self.placed.digit(UNIQUE_PIECE_COUNT) == 0
        {
            return out;
        }

        let mut seen = HashSet::new();
        let mut todo = vec![Layer {
            placed: Pieces(0),
            remaining: self.remaining}];

        while let Some(t) = todo.pop() {
            if seen.contains(&t) {
                continue;
            }

            for i in 0..10 {
                if t.remaining.digit(i) > 0 {
                    let next = t.place(i);
                    if next.placed.area() <= self.placed.area() {
                        todo.push(next);
                    }
                }
            }

            seen.insert(t);
        }
        seen.into_iter().collect()
    }

    /*
     *  Moves a piece from the set of remaning pieces to
     *  the set of placed pieces, returning a new Layer.
     */
    fn place(&self, i: usize) -> Layer {
        Layer { placed: self.placed.add(i),
                remaining: self.remaining.take(i) }
    }
}

////////////////////////////////////////////////////////////////////////////////

//  Encodes an X position, Y position, and rotation (4 options) into 2 bytes
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Position {
    _x: u8,
    _y: u8,
}

impl Position {
    fn new(x: i32, y: i32, r: u8) -> Position {
        debug_assert!(x >= 0 && x < 127);
        debug_assert!(y >= 0 && y < 127);
        debug_assert!(r < 4);
        Position { _x: ((r & 1) << 7) | (x as u8),
                   _y: ((r & 2) << 6) | (y as u8) }
    }

    fn empty() -> Position {
        Position { _x: 0xFF, _y: 0xFF }
    }

    fn is_empty(&self) -> bool {
        self._x == 0xFF && self._y == 0xFF
    }

    fn x(&self) -> i32 { (self._x & 0x7F) as i32 }
    fn y(&self) -> i32 { (self._y & 0x7F) as i32 }
    fn r(&self) -> u8 { ((self._x & 0x80) >> 7) | ((self._y & 0x80) >> 6) }

    fn shift(&self, dx: i32, dy: i32) -> Position {
        Position::new(self.x() + dx, self.y() + dy, self.r())
    }
}

//  Encodes a full set of pieces, placed on a single 2D layer
#[derive(Eq, PartialEq, Clone, Debug, Copy, Hash)]
pub struct Placement([Position; 20]);

impl Placement {
    fn new() -> Placement {
        Placement([Position::empty(); 20])
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|p| p.is_empty())
    }

    fn place(&self, i: u8, x: i32, y: i32, r: u8) -> Placement {
        let mut out = self.clone();

        let dx = max(-x, 0);
        let dy = max(-y, 0);
        if dx != 0 || dy != 0 {
            for p in &mut out.0 {
                if !p.is_empty() {
                    *p = p.shift(dx, dy);
                }
            }
        }

        let pos = Position::new(x + dx, y + dy, r);
        let index = (2 * i) as usize;
        if out.0[index].is_empty() {
            out.0[index] = pos;
        } else {
            debug_assert!(out.0[index + 1].is_empty());
            out.0[index + 1] = pos;
        }
        return out;
    }

    /*
     *  Attempts to place a place a new piece on the same layer as this
     *  placement, returning the resulting placement on success.
     */
    fn try_place(&self, i: u8, x: i32, y: i32, r: u8) -> Option<Placement> {
        debug_assert!(r < 4);
        debug_assert!(i < 10);

        let piece = ((i * 4) + r) as usize;

        let mut got_neighbor = false;
        for p in self.0.iter() {
            let r = OVERLAP_TABLES.at(piece).at(p.x() - x, p.y() - y, i as usize, r as usize); // TODO: remove usizes
            match r {
                Overlap::_Partial(_) => panic!("Uncleaned index"),
                Overlap::None => (),
                Overlap::Neighbor => got_neighbor = true,
                Overlap::Partial(t) => return None,
                Overlap::Full => return None,
            }
        }

        if got_neighbor {
            return Some(self.place(i, x, y, r));
        } else {
            return None;
        }

    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Graph(HashMap<Layer, Vec<Layer>>);

impl Graph {
    pub fn build() -> Graph {

        let _boop = possible_overlaps();

        let timer = SystemTime::now();
        let mut out = Graph(HashMap::new());

        let mut todo = Vec::new();
        todo.push(Layer::base());

        while todo.len() > 0 {
            println!("Running with {} to do ({:?})", todo.len(), timer.elapsed());
            let mut next = Vec::new();
            for t in todo {
                if out.0.contains_key(&t) {
                    continue;
                }
                let e = t.next();
                for n in e.iter() {
                    next.push(n.clone());
                }
                out.0.insert(t, e);
            }
            todo = next;
        }

        out
    }
}

fn possible_overlaps() -> Vec<(Pieces, Pieces)> {
    let mut out = Vec::new();
    let mut pieces = HashSet::new();

    for i in 0..(3 as u16).pow(10) {
        println!("{}", i);
        let mut d: Vec<(Pieces, Pieces)> = (0..(3 as u16).pow(10))
            .into_par_iter()
            .filter(|j| {
                let a = Pieces(i);
                let b = Pieces(*j);

                if a.len() < 2 { return false; }
                if a.area() < b.area() { return false; }

                for q in 0..10 {
                    if a.digit(q) + b.digit(q) > 2 {
                        return false;
                    }
                }
                return true; })
            .map(|j| { (Pieces(i), Pieces(j)) } )
            .collect();
        out.append(&mut d);
    }


    for (a, b) in out.iter() {
        pieces.insert(a.clone());
        pieces.insert(b.clone());
    }

    for p in pieces.iter() {
        println!("{:?}", p);
    }

    println!("Got {} possible overlaps of {} pieces", out.len(), pieces.len());
    return out;
}
