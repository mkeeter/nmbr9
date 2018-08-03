use std::collections::HashSet;
use std::fmt;
use std::char::from_digit;
use std::collections::HashMap;
use std::time::SystemTime;
use std::cmp::max;
use std::sync::{Mutex, RwLock};

use rayon::prelude::*;
use colored::*;

use piece::{Piece, PIECES, PIECE_COLORS, PIECE_AREA, UNIQUE_PIECE_COUNT, Overlap};
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

        // Special case for the first piece being placed
        if self.is_empty() {
            if x == 0 && y == 0 {
                return Some(self.place(i, x, y, r));
            } else {
                return None;
            }
        }

        let mut got_neighbor = false;
        for (n, p) in self.0.iter().enumerate().filter(|(_, p)| !p.is_empty()) {
            let index = (n / 2) * 4 + (p.r() as usize);
            let r = OVERLAP_TABLES.at(index).at(p.x() - x, p.y() - y, r, i);
            match r {
                Overlap::_Partial(_) => panic!("Uncleaned index"),
                Overlap::None => (),
                Overlap::Neighbor => got_neighbor = true,
                Overlap::Partial(_) => return None,
                Overlap::Full => return None,
            }
        }

        if got_neighbor {
            return Some(self.place(i, x, y, r));
        } else {
            return None;
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.0.iter()
            .filter(|p| !p.is_empty())
            .fold((0, 0), |(x, y), p| (max(x, p.x() + 4), max(y, p.y() + 4)))
    }

    pub fn pretty_print(&self) {
        let (w, h) = self.size();

        let mut v = vec![-1; (w * h) as usize];

        for (i, a) in self.0.iter().enumerate().filter(|(_, a)| !a.is_empty()) {
            let p = Piece::from_u16(PIECES[i / 2]).rotn(a.r());
            for (px, py) in p.pts {
                let x = px + a.x();
                let y = py + a.y();
                v[(x + y * w) as usize] = (i / 2) as i32;
            }
        }

        for y in (0..h).rev() {
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
        print!("\n");
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Graph(HashMap<Layer, Vec<Layer>>);

impl Graph {
    pub fn build() -> Graph {

        let _beep = possible_placements();
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

////////////////////////////////////////////////////////////////////////////////

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

fn possible_placements() -> Vec<Placement> {
    let seen: RwLock<HashMap<Pieces, HashSet<Placement>>> = RwLock::new(HashMap::new());

    let mut todo = Vec::new();
    todo.push((Placement::new(),
               Layer { placed: Pieces::empty(), remaining: Pieces::all() }));

    let timer = SystemTime::now();

    while todo.len() > 0 {
        // Skip all placements that have already been seen
        let next: Vec<_> = todo.into_par_iter()
            .filter(|(placement, layer)| seen.read()
                                             .unwrap()
                                             .get(&layer.placed)
                                             .map(|h| !h.contains(&placement))
                                             .unwrap_or(true))
            .map(|(placement, layer)| {
                // Insert a brand new HashSet if this is the first time we've
                // seen this particular set of pieces
                if !seen.read().unwrap().contains_key(&layer.placed) {
                    let mut h = HashSet::new();
                    h.insert(placement);
                    seen.write().unwrap().insert(layer.placed, h);
                // Otherwise, add this particular arrangement
                } else {
                    let i = seen.write().unwrap().get_mut(&layer.placed)
                                .unwrap().insert(placement);
                    debug_assert!(!i);
                }


                let mut next = Vec::new();

                // Iterate over all of the possible pieces, rotations,
                // and positions, checking to see which placements are valid
                let size = placement.size();
                for i in (0..10).filter(|i| { layer.remaining.digit(*i) > 0 }) {
                    for r in 0..4 {
                        for x in -4..=size.0 + 4 {
                            for y in -4..=size.1 + 4 {
                                if let Some(p) = placement.try_place(i as u8, x, y, r) {
                                    next.push((p, layer.place(i)));
                                    /*
                                    for (i, q) in p.0.iter().enumerate() {
                                        if !q.is_empty() {
                                            println!("i: {}: x = {}, y = {}, r = {}",
                                                   i, q.x(), q.y(), q.r());
                                        }
                                    }
                                    p.pretty_print();
                                    */
                                }
                            }
                        }
                    }
                }
                return next;
            })
            .reduce(|| Vec::new(), |mut a, mut v| { a.append(&mut v); return a});
        todo = next;
        println!("Got {} possible placements in {:?}", todo.len(), timer.elapsed());
    }

    return Vec::new();
}
