use piece::{UNIQUE_PIECE_COUNT, MAX_ROTATIONS};

#[derive(Clone, Debug)]
pub struct Bag {
    data: [usize; UNIQUE_PIECE_COUNT],
}

impl Bag {
    fn new() -> Bag {
        Bag { data: [0; UNIQUE_PIECE_COUNT] }
    }

    // Interprets an integer as a ternary number
    // that tells us how many of each piece we put
    // into the bag.
    pub fn from_usize(mut p: usize) -> Bag {
        let mut out = Bag::new();
        for i in 0..UNIQUE_PIECE_COUNT {
            out.data[i] = p % 3;
            p /= 3;
        }
        return out;
    }

    pub fn as_usize(&self) -> usize {
        let mut p = 0;
        for i in (0..UNIQUE_PIECE_COUNT).rev() {
            p *= 3;
            p += self.data[i];
        }
        return p;
    }

    pub fn len(&self) -> usize {
        self.data.iter().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn take(&self, id: usize) -> Bag {
        let mut out = self.clone();
        let index = id / MAX_ROTATIONS;
        if out.data[index] == 0 {
            panic!("Attempted to remove non-existent piece");
        } else {
            out.data[index] -= 1;
        }
        return out;
    }

    pub fn contains(&self, other: &Bag) -> bool {
        for i in 0..UNIQUE_PIECE_COUNT {
            if other.data[i] > self.data[i] {
                return false;
            }
        }
        return true;
    }

    // Returns the flat score of all the tiles in the bag
    // (i.e. the sum of their face values)
    pub fn score_flat(&self) -> usize {
        let mut s = 0;
        for i in 0..UNIQUE_PIECE_COUNT {
            s += self.data[i] * i;
        }
        return s;
    }

    // Returns an upper bound on score if the tiles are stacked
    // This has the conservative assumption that each layer is a mere
    // two tiles (the minimum to stack onto).
    pub fn score_stacked(&self) -> usize {
        let mut remaining = self.len();
        let mut score = 0;
        for p in (0..UNIQUE_PIECE_COUNT).rev() {
            for _ in 0..self.data[p] {
                score += p * (((remaining + 1) / 2) - 1);
                remaining -= 1;
            }
        }
        debug_assert!(remaining == 0);
        return score;
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

pub struct BagIterator<'a> {
    bag: &'a Bag,
    i: usize,
    r: usize,
}

impl<'a> Iterator for BagIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        while self.i < UNIQUE_PIECE_COUNT && self.bag.data[self.i] == 0 {
            self.i += 1;
        }
        if self.i == UNIQUE_PIECE_COUNT {
            return None;
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

    #[test]
    fn len() {
        let b = Bag::from_usize(0);
        assert_eq!(b.len(), 0);

        let b = Bag::from_usize(1);
        assert_eq!(b.len(), 1);

        let b = Bag::from_usize(2);
        assert_eq!(b.len(), 2);

        let b = Bag::from_usize(3);
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn take() {
        let b = Bag::from_usize(1);
        let b = b.take(3);
        assert_eq!(b.len(), 0);
    }

    #[test]
    #[should_panic]
    fn bad_take() /* Hi Twitter! */ {
        let b = Bag::from_usize(1);
        let b = b.take(4);
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn iter() {
        let b = Bag::from_usize(0);
        let mut i = b.into_iter();
        assert_eq!(i.next(), None);

        let b = Bag::from_usize(1);
        let mut i = b.into_iter();
        assert_eq!(i.next(), Some(0));
        assert_eq!(i.next(), Some(1));
        assert_eq!(i.next(), Some(2));
        assert_eq!(i.next(), Some(3));
        assert_eq!(i.next(), None);

        let b = Bag::from_usize(2);
        let mut i = b.into_iter();
        assert_eq!(i.next(), Some(0));
        assert_eq!(i.next(), Some(1));
        assert_eq!(i.next(), Some(2));
        assert_eq!(i.next(), Some(3));
        assert_eq!(i.next(), None);

        let b = Bag::from_usize(3);
        let mut i = b.into_iter();
        assert_eq!(i.next(), Some(4));
        assert_eq!(i.next(), Some(5));
        assert_eq!(i.next(), Some(6));
        assert_eq!(i.next(), Some(7));
        assert_eq!(i.next(), None);
    }

    #[test]
    fn as_usize() {
        for i in 0..3_usize.pow(10) {
            let b = Bag::from_usize(i);
            assert_eq!(b.as_usize(), i);
        }
    }

    #[test]
    fn score_flat() {
        assert_eq!(Bag::from_usize(0).score_flat(), 0);
        assert_eq!(Bag::from_usize(1).score_flat(), 0);
        assert_eq!(Bag::from_usize(2).score_flat(), 0);
        assert_eq!(Bag::from_usize(3).score_flat(), 1);
        assert_eq!(Bag::from_usize(4).score_flat(), 1);
        assert_eq!(Bag::from_usize(5).score_flat(), 1);
        assert_eq!(Bag::from_usize(6).score_flat(), 2);
        assert_eq!(Bag::from_usize(7).score_flat(), 2);
        assert_eq!(Bag::from_usize(8).score_flat(), 2);
        assert_eq!(Bag::from_usize(9).score_flat(), 2);
    }

    #[test]
    fn score_stacked() {
        assert_eq!(Bag::from_usize(0).score_stacked(), 0);
        assert_eq!(Bag::from_usize(1).score_stacked(), 0);
        assert_eq!(Bag::from_usize(2).score_stacked(), 0);
        assert_eq!(Bag::from_usize(3).score_stacked(), 0);
        assert_eq!(Bag::from_usize(4).score_stacked(), 0);
        assert_eq!(Bag::from_usize(5).score_stacked(), 1);
        assert_eq!(Bag::from_usize(6).score_stacked(), 0);
        assert_eq!(Bag::from_usize(7).score_stacked(), 1);
        assert_eq!(Bag::from_usize(8).score_stacked(), 2);
        assert_eq!(Bag::from_usize(9).score_stacked(), 0);
    }
}
