use piece::UNIQUE_PIECE_COUNT;
use bag::Bag;
use state::State;

pub struct Results {
    // For a particular set of pieces (represented by a 10-digit ternary value),
    // what is the highest possible score (if we start with the pieces placed
    // on a flat, empty table)?
    scores: Vec<(usize, usize)>,
}

impl Results {
    pub fn new() -> Results {
        Results {
            scores: vec![(0, 0); 3_usize.pow(UNIQUE_PIECE_COUNT as u32)],
        }
    }

    // Returns an upper bound score for a given state, with a certain number
    // of pieces remaining in the bag to be placed.
    pub fn upper_score_bound(&self, bag: &Bag, state: &State) -> usize {
        let b = bag.as_usize();

        let (available_score, available_delta) = self.scores[b];

        let layers = state.layers();
        return available_score + (layers + 1) * available_delta;
    }

    pub fn write_score(&mut self, target: usize, score: usize) {
        self.scores[target] = (score, Bag::from_usize(target).score());
    }
}
