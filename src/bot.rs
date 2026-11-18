use crate::action_inputs::Action;
use crate::game::{Game, GameState};
use crate::load_resources::load_resources;

use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub trait SinglePlayerBot {
    /// Create a new bot given the path to the resources dir.
    fn new(config: &Path) -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(&self) -> Action;

    /// Updates the current bot state assuming the player plays a given action.
    fn update_with(&mut self, action: Action);
}

pub struct MySinglePlayerBot {
    game: Game,
}

const DISCOUNT_FACTOR: f64 = 0.95;
const HOLE_COST: f64 = 20.0;
const FLATNESS_COST: f64 = 1.0;
const HEIGHT_COST: f64 = 1.0;
const LINE_CLEAR_REWARD: f64 = 100.0;

impl MySinglePlayerBot {
    fn landing_reward(state: GameState, game: &Game, moves_to_target: usize) -> f64 {
        match state {
            GameState::Locked { rows_cleared } => {
                let board = &game.board;
                let board_cost = HOLE_COST * board.hole_count() as f64
                    + FLATNESS_COST * board.flatness_cost() as f64
                    + HEIGHT_COST * board.highest_filled_row().unwrap_or(0) as f64;
                let discount = DISCOUNT_FACTOR.powi(moves_to_target as i32);
                discount * (rows_cleared as f64 * LINE_CLEAR_REWARD) - board_cost
            }
            GameState::ToppedOut => -1000.0,
            GameState::Playing => unreachable!("hard drop always locks the piece"),
        }
    }

    /// Find the best landing among positions reachable by left/right moves and
    /// rotations at the piece's current height. The queue also gives a shortest
    /// sequence of inputs to each setup position.
    fn best_drop(&self) -> Option<(Vec<Action>, f64)> {
        let start = self.game.board.clone();
        start.current_piece?;
        let start_y = start.current_piece_position[1];
        let mut queue = VecDeque::from([(start.clone(), Vec::<Action>::new())]);
        let mut visited = HashSet::from([(
            start.current_piece_position[0],
            start.current_piece_rotation_state_idx,
        )]);
        let mut best: Option<(Vec<Action>, f64)> = None;

        while let Some((board, path)) = queue.pop_front() {
            let mut dropped = self.game.clone();
            dropped.board = board.clone();
            if let Ok(state) = dropped.update(Action::HardDrop) {
                let score = Self::landing_reward(state, &dropped, path.len());
                if best.as_ref().is_none_or(|(best_path, best_score)| {
                    score > *best_score || (score == *best_score && path.len() < best_path.len())
                }) {
                    best = Some((path.clone(), score));
                }
            }

            for action in [
                Action::MoveLeft,
                Action::MoveRight,
                Action::RotateClockwise,
                Action::RotateCounterClockwise,
            ] {
                let mut next = board.clone();
                let moved = match action {
                    Action::MoveLeft => next.move_current_piece(-1, 0),
                    Action::MoveRight => next.move_current_piece(1, 0),
                    Action::RotateClockwise => next.rotate_current_piece(true),
                    Action::RotateCounterClockwise => next.rotate_current_piece(false),
                    _ => unreachable!(),
                };
                if !moved || next.current_piece_position[1] != start_y {
                    continue;
                }
                let key = (
                    next.current_piece_position[0],
                    next.current_piece_rotation_state_idx,
                );
                if visited.insert(key) {
                    let mut next_path = path.clone();
                    next_path.push(action);
                    queue.push_back((next, next_path));
                }
            }
        }

        best
    }
}

impl SinglePlayerBot for MySinglePlayerBot {
    fn new(config: &Path) -> Self {
        let (config, tetrominoes) = load_resources(config).unwrap();
        Self {
            game: Game::new(config, tetrominoes).unwrap(),
        }
    }

    fn get_move(&self) -> Action {
        if let Some((mut path, _)) = self.best_drop() {
            path.push(Action::HardDrop);
            println!("{path:?}");
            path[0]
        } else {
            Action::None
        }
    }

    fn update_with(&mut self, action: Action) {
        sleep(Duration::from_millis(250));
        self.game.update(action).expect("Failed to update.");
    }
}
