use crate::resources::Tetromino;
use rand::Rng;
use rand_xoshiro::Xoshiro128PlusPlus;
use rand::{seq::SliceRandom, SeedableRng};

type RngAlgorithm = Xoshiro128PlusPlus;

pub struct PieceGenerator {
    bag: [Tetromino; 7],
    idx: usize,
    rng: RngAlgorithm
}

impl PieceGenerator {
    pub fn new(
        mut bag: [Tetromino; 7],
        seed: <RngAlgorithm as SeedableRng>::Seed
    ) -> Self {
        let mut rng = RngAlgorithm::from_seed(seed);
        bag.shuffle(&mut rng);
        Self { idx: 0, bag, rng }
    }

    // for getting a new piece, update randomized iter
    pub fn next(&mut self) {
        self.idx += 1;
        if self.idx >= self.bag.len() {
            self.bag.shuffle(&mut self.rng); // 
            // start over, s.t. we get index 0 again
            self.idx = 0;
        }
    }

    // then call cur to get the piece
    pub fn cur(&self) -> &Tetromino {
        // remember that bag contains the Tetromino pieces
        &self.bag[self.idx]
    }
}