use std::collections::HashSet;

use piece::{PIECE_AREA, UNIQUE_PIECE_COUNT};

/*
 *  Defines the number of pieces on each of 10 maximum layers
 */
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Stackup([usize; 10]);

/*
 *  Generates all of the valid stackups that use all 20 pieces
 */
impl Stackup {
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
        return seen.into_iter().collect();
    }
}
