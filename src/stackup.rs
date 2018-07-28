use std::collections::HashSet;
use std::fmt;
use std::char::from_digit;
use std::mem::size_of;

use piece::{PIECE_AREA, UNIQUE_PIECE_COUNT};

/*
 *  Defines the number of pieces on each of 10 maximum layers
 */
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Stackup([usize; 10]);

/*
 *  Represents a selection of which pieces to put on which layer
 *  Each layer stores a 10-digit ternary value, where each digit
 *  means placing 0, 1, or 2 pieces of that digit's value.
 */
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Layers([Layer; 10]);

/*  Represents a single layer as a ternary value */
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct Layer(u16);

impl fmt::Debug for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Layer(")?;
        for i in (0..10).rev() {
            write!(f, "{}", from_digit(
                    ((self.0 / (3 as u16).pow(i)) % 3) as u32, 3).unwrap())?;
        }
        return write!(f, ")");
    }
}

impl Layer {
    fn digit(&self, i: usize) -> u16 {
        (self.0 / Layer::unit(i)) % 3
    }

    fn take(&self, i: usize) -> Layer {
        debug_assert!(self.digit(i) > 0);
        Layer(self.0 - Layer::unit(i))
    }

    fn add(&self, i: usize) -> Layer {
        debug_assert!(self.digit(i) < 2);
        Layer(self.0 + Layer::unit(i))
    }

    fn unit(i: usize) -> u16 {
        (3 as u16).pow(i as u32)
    }

    fn area(&self) -> u32 {
        let mut out = 0;
        for i in 0..10 {
            out += PIECE_AREA[i] * (self.digit(i) as u32);
        }
        return out;
    }

    fn choose_(&self, n: usize, seen: &mut HashSet<Layer>) -> HashSet<Layer> {
        let mut out = HashSet::new();
        if n == 0 {
            out.insert(Layer(0));
            return out;
        }
        else if seen.contains(self) {
            return out;
        }
        seen.insert(self.clone());

        for i in 0..10 {
            if self.digit(i) > 0 {
                for o in self.take(i).choose_(n - 1, seen) {
                    out.insert(o.add(i));
                }
            }
        }
        return out;
    }

    pub fn choose(&self, n: usize) -> HashSet<Layer> {
        return self.choose_(n, &mut HashSet::new());
    }
}


/*
 *  Generates all of the valid stackups that use all 20 pieces
 */
impl Stackup {
    pub fn to_layers(&self) -> Vec<Layers> {

        let mut todo = Vec::new();
        todo.push((Layers([Layer(0); 10]), Layer((3 as u16).pow(10) - 1)));

        println!("Building layers for {:?}", self);
        let mut discarded = 0;
        for i in 0..9 {
            println!("i: {} (got {} todo, {} discarded)", i, todo.len(), discarded);
            discarded = 0;
            let mut next = Vec::new();
            for (arr, rem) in todo.iter() {

                for d in rem.choose(self.0[i]) {
                    // Discard any stackups that violate the area constraint
                    if i > 0 && d.area() > arr.0[i - 1].area() {
                        discarded += 1;
                        continue;
                    }
                    let mut arr = arr.clone();
                    arr.0[i] = d;
                    next.push((arr, Layer(rem.0 - d.0)));
                }
            }
            todo = next;
        }

        println!("Found {} layer combinations taking {} MB", todo.len(),
                 todo.len() * size_of::<Layers>() / 1024 / 1024);
        // Unpack the final layer of each stackup
        todo.iter().map(|(a, rem)| {
            let mut a = a.clone();
            a.0[9] = *rem;
            return a; }).collect()
    }

    pub fn gen() -> Vec<Stackup> {
        let mut todo = Vec::new();

        {   // Construct a starting point, with all tiles at ground level
            let mut start = Stackup([0; 10]);
            start.0[0] = 20;
            todo.push(start);
        }

        let mut areas = [0; UNIQUE_PIECE_COUNT * 2];
        for (i, a) in PIECE_AREA.iter().enumerate() {
            areas[i * 2] = *a;
            areas[i * 2 + 1] = *a;
        }
        areas.sort();

        let mut seen = HashSet::new();
        while let Some(target) = todo.pop() {
            if seen.contains(&target) {
                continue;
            }

            // Discard invalid arrangements
            // (which have a one-piece layer supporting anything)
            if (1..10).any(|i| target.0[i] > 0 && target.0[i-1] < 2) {
                continue;
            }

            // Discard invalid arrangment which have pairwise impossible
            // area constraints (i.e. even in the best conditions, it's
            // impossible to have the area of the higher layer be <= the
            // lower layer's area).
            if (1..10).any(|i| {
                    let upper: u32 = areas[0..target.0[i]].iter().sum();
                    let lower: u32 = areas[20-target.0[i-1]..20].iter().sum();
                    return upper > lower;}) {
                continue;
            }
            seen.insert(target.clone());

            for i in 0..10 {
                if target.0[i] == 0 {
                    break;
                }
                for j in (i + 1)..10 {
                    if target.0[j - 1] == 0 {
                        break;
                    }
                    let mut next = target.clone();
                    next.0[i] -= 1;
                    next.0[j] += 1;
                    todo.push(next);
                }
            }
        }

        println!("Found {} arrangements", seen.len());
        for s in &seen {
            println!("Got seen {:?}", s);
        }

        let out: Vec<Stackup> = seen.into_iter().collect();
        for i in 0..10 {
            out[i].to_layers();
        }
        return out;
    }
}
