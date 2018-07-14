use piece::{UNIQUE_PIECE_COUNT, MAX_ROTATIONS};

#[derive(Clone)]
struct Bag {
    data: [usize; UNIQUE_PIECE_COUNT],
}

impl Bag {
    fn new() -> Bag {
        Bag { data: [0; UNIQUE_PIECE_COUNT] }
    }

    // Interprets an integer as a ternary number
    // that tells us how many of each piece we put
    // into the bag.
    fn from_usize(mut p: usize) -> Bag {
        let mut out = Bag::new();
        for i in 0..UNIQUE_PIECE_COUNT {
            out.data[i] = p % 3;
            p /= 3;
        }
        return out;
    }

    fn len(&self) -> usize {
        self.data.iter().sum()
    }

    fn take(&self, id: usize) -> Bag {
        let mut out = self.clone();
        out.data[id / MAX_ROTATIONS] -= 1;
        return out;
    }
}

impl<'a> IntoIterator for &'a Bag {
    type Item = usize;
    type IntoIter = BagIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        BagIterator { bag: self, i: 0, r: 0 }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct BagIterator<'a> {
    bag: &'a Bag,
    i: usize,
    r: usize,
}

impl<'a> Iterator for BagIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        while self.bag.data[self.i] == 0
        {
            self.i += 1;
            if self.i == UNIQUE_PIECE_COUNT {
                return None;
            }
        }

        let out = self.i * MAX_ROTATIONS + self.r;

        self.r += 1;
        if self.r == MAX_ROTATIONS
        {
            self.r = 0;
            self.i += 1;
        }

        return Some(out);
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_usize() {
        let b = Bag::from_usize(0);
        for i in 0..UNIQUE_PIECE_COUNT {
            assert_eq!(b.data[i], 0);
        }

        let b = Bag::from_usize(1);
        assert_eq!(b.data[0], 1);
        for i in 1..UNIQUE_PIECE_COUNT {
            assert_eq!(b.data[i], 0);
        }

        let b = Bag::from_usize(2);
        assert_eq!(b.data[0], 2);
        for i in 1..UNIQUE_PIECE_COUNT {
            assert_eq!(b.data[i], 0);
        }

        let b = Bag::from_usize(3);
        assert_eq!(b.data[0], 0);
        assert_eq!(b.data[1], 1);
        for i in 2..UNIQUE_PIECE_COUNT {
            assert_eq!(b.data[i], 0);
        }
    }
}
