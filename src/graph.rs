use std::collections::HashSet;
use std::fmt;
use std::char::from_digit;
use std::collections::HashMap;
use std::time::SystemTime;

use rayon::prelude::*;

use piece::{PIECE_AREA, UNIQUE_PIECE_COUNT};

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
                    if a.digit(q) + b.digit(q) <= 2 {
                        return false;
                    }
                }
                return true; })
            .map(|j| { (Pieces(i), Pieces(j)) } )
            .collect();
        out.append(&mut d);
    }
    println!("Got {} possible overlaps", out.len());
    return out;
}
