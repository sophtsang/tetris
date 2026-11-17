// manages information from both players or bots in multiplayer

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro128PlusPlus;
use std::collections::VecDeque;
use crate::resources::Config;
use crate::game::{Game, GameState};

type RngAlgorithm = Xoshiro128PlusPlus;

// keeps track of p1 and p2's amount of cleared rows and accum garbage
pub struct GameLog {
    config: Config,
    pub num_players: usize,
    pub game_states: Vec<GameState>, // tracks whethr the player has topped out or not
    pub cleared_rows: Vec<usize>, // both are indexed by player idx
    pub garbage_rows: Vec<VecDeque<Garbage>>, // newest at front, oldest at back
    rngs: Vec<RngAlgorithm> // for rand col generation
}

impl GameLog {
    pub fn new(config: Config, num_players: usize) -> Self {
        Self {
            config: config,
            num_players: num_players,
            game_states: vec![GameState::Playing; num_players],
            cleared_rows: vec![0; num_players],
            garbage_rows: vec![VecDeque::new(); num_players],
            rngs: vec![RngAlgorithm::from_seed(config.random_seed); num_players]
        }
    }

    // player is dead
    pub fn topped_out(&mut self, player: usize) {
        self.cleared_rows[player] = 0;
        self.game_states[player] = GameState::ToppedOut;
    }

    // update cleared_rows ONLY WHEN PIECE IS LOCKED
    pub fn update_cleared(&mut self, player: usize, cleared: usize) {
        self.cleared_rows[player] = cleared.saturating_sub(1); // readme wants this
        self.game_states[player] = GameState::Locked{ rows_cleared: cleared };
    }

    /// total garbage in player's queue prior to removal or addition
    fn pending(&self, player: usize) -> usize {
        self.garbage_rows[player].iter().map(|g| g.num_rows_to_create).sum()
    }

    // specifically the removal and addition of garbage
    pub fn exchange_garbage(&mut self) {
        // for reference these are the steps that the readme gives for updating garbage
        // g1, g2: pending garbage before this frame's changes
        let pending = [self.pending(0), self.pending(1)];

        // 1) and 2) first remove and add new garbage based on cleared_rows
        for idx in 0..2 {
            let opp = 1 - idx;
            let sent = self.cleared_rows[idx];

            let mut remove = sent.min(pending[idx]);
            while remove > 0 {
                let oldest = self.garbage_rows[idx].back_mut().unwrap();
                let n = remove.min(oldest.num_rows_to_create); // capped by num garbage in oldest
                oldest.num_rows_to_create -= n;
                remove -= n;
                if oldest.num_rows_to_create == 0 {
                    self.garbage_rows[idx].pop_back();
                }
            }

            // leftover cleared rows that haven't been used to clear garbage -> go to enemy
            let remainder = sent.saturating_sub(pending[idx]);
            if remainder > 0 {
                let timer = self.config.garbage_queue_time as usize;
                self.garbage_rows[opp].push_front(Garbage::new(remainder, timer));
            }
        }
    }

    // involves:
    //  1) removal of own garbage based off of number of cleared rows this round,
    //  2) throwing garbage at the enemy based of remaining cleared rows after own removal
    //  3) updating the board with remaining Garbage if individual timers == 0
    //  4) decrementing timers for invidual Garbage
    //  5) clearing cleared_rows
    pub fn update(&mut self, games: &mut [Game]) -> Result<(), String> {
        if self.num_players != 2 {
            return Ok(());
        }
        // 1) and 2)
        self.exchange_garbage();

        // 3) draw garbage on board if timer = 0 and piece is locked
        for idx in 0..2 {
            if !matches!(self.game_states[idx], GameState::Locked { .. }) {
                continue; // only a player who locked a piece this frame gets garbage
            }
            // apply every due block where timer is 0, oldest first, each with its own random column
            while self.garbage_rows[idx].back()
                .is_some_and(|g| g.frames_till_applied == 0)
            {
                let rows = self.garbage_rows[idx].pop_back().unwrap().num_rows_to_create;
                let empty_col = (self.rngs[idx].next_u32() % self.config.width) as usize;
                if games[idx].generate_garbage(rows, empty_col)? == GameState::ToppedOut {
                    self.game_states[idx] = GameState::ToppedOut;
                    break; // oops, topped out: no further garbage for this player
                }
            }
            if self.game_states[idx] != GameState::ToppedOut {
                self.game_states[idx] = GameState::Playing; // back to normal
            }
        }
        // 4) decrement timers, floor is 0
        for idx in 0..2 {
            for g in self.garbage_rows[idx].iter_mut() {
                g.frames_till_applied = g.frames_till_applied.saturating_sub(1);
            }
        }
        // 5) in case player tops out, doesn't just keep removing rows
        self.cleared_rows.fill(0);
        Ok(())
    }
}

// represents the amount of garbage generated after 1 update round
// will also be the amount of garbage drawn onto the board if timer hits 0
#[derive(Clone)]
pub struct Garbage {
    pub num_rows_to_create: usize,
    pub frames_till_applied: usize
}

impl Garbage {
    // need width to determine range of cols in PieceGenerator
    pub fn new(num_rows: usize, garbage_queue_time: usize) -> Self {
        Self {
            num_rows_to_create: num_rows,
            frames_till_applied: garbage_queue_time
        }
    }
}