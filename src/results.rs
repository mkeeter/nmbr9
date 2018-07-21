use piece::UNIQUE_PIECE_COUNT;
use bag::Bag;
use state::State;

pub struct Results {
    // For a particular set of pieces (represented by a 10-digit ternary value),
    // what is the highest possible score (if we start with the pieces placed
    // on a flat, empty table)?
    scores: Vec<Option<usize>>,

    // For a particular set of pieces, how much does the score go up if we
    // place them a layer higher?
    deltas: Vec<usize>
}

impl Results {
    pub fn new() -> Results {
        Results {
            scores: vec![None; 3_usize.pow(UNIQUE_PIECE_COUNT as u32)],
            deltas: (0..3_usize.pow(UNIQUE_PIECE_COUNT as u32)).map(
                |i| Bag::from_usize(i).score_flat()).collect(),
        }
    }

    // Returns an upper bound score for a given state, with a certain number
    // of pieces remaining in the bag to be placed.
    pub fn upper_score_bound(&self, bag: &Bag, state: &State) -> usize {
        let layers = state.layers();
        let b = bag.as_usize();
        let available_delta = self.deltas[b];

        let score = if let Some(available_score) = self.scores[b] {
            available_score
        } else {
            bag.score_stacked()
        };
        return score + (layers + 1) * self.deltas[b];
    }

    pub fn write_score(&mut self, target: usize, score: usize) {
        self.scores[target] = Some(score);
    }
}
