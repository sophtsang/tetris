use crate::action_inputs::Action;
use crate::game::{Game, GameState};
use crate::load_resources::load_resources;

use std::path::Path;

pub trait SinglePlayerBot {
    /// Create a new bot given the path to the resources dir.
    fn new(config: &Path) -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(&self) -> Action;

    /// Updates the current bot state assuming the player plays a given action.
    /// If the bot is playing, this will likely be the result of
    /// `bot.get_move()`.
    fn update_with(&mut self, action: Action);
}

pub struct MySinglePlayerBot {
    game: Game,
}

const DISCOUNT_FACTOR: f64 = 0.95;
const HOLE_COST: f64 = 10.0;
const FLATNESS_COST: f64 = 1.0;
const HEIGHT_COST: f64 = 1.0;

impl MySinglePlayerBot {
    fn reward(game_state: &GameState, depth: u32, game_after_action: &Game) -> f64 {
        let discount = DISCOUNT_FACTOR.powi(depth as i32);

        match game_state {
            GameState::Locked { rows_cleared } => {
                let board = &game_after_action.board;
                let board_cost = HOLE_COST * board.hole_count() as f64
                    + FLATNESS_COST * board.flatness_cost() as f64
                    + HEIGHT_COST * board.highest_filled_row().unwrap_or(0) as f64;
                discount * (*rows_cleared as f64 * 100.0) - board_cost
            }
            GameState::Playing => 0.0,
            GameState::ToppedOut => -1000.0,
        }
    }

    fn tree_search(remaining_layers: u32, game: Game) -> Option<(Vec<Action>, f64)> {
        let mut best_seq = None;

        for action in [
            Action::HardDrop,
            Action::MoveLeft,
            Action::MoveRight,
            Action::RotateClockwise,
            Action::RotateCounterClockwise,
            Action::MoveDown,
            Action::None,
        ] {
            let mut game_after_action = game.clone();

            if let Ok(game_state) = game_after_action.update(action) {
                let move_reward = MySinglePlayerBot::reward(
                    &game_state,
                    6 - remaining_layers,
                    &game_after_action,
                );

                if game_state == GameState::Playing && remaining_layers > 0 {
                    let optimal_continuation =
                        MySinglePlayerBot::tree_search(remaining_layers - 1, game_after_action);

                    if let Some((mut actions, reward)) = optimal_continuation {
                        actions.push(action);

                        match best_seq {
                            None => best_seq = Some((actions, move_reward + reward)),
                            Some((_, best_reward)) => {
                                if move_reward + reward > best_reward {
                                    best_seq = Some((actions, move_reward + reward))
                                }
                            }
                        }
                    }
                } else {
                    match best_seq {
                        None => best_seq = Some((vec![action], move_reward)),
                        Some((_, best_reward)) => {
                            if move_reward > best_reward {
                                best_seq = Some((vec![action], move_reward))
                            }
                        }
                    }
                }
            }
        }

        best_seq
    }
}

impl SinglePlayerBot for MySinglePlayerBot {
    fn new(config: &Path) -> Self {
        let (config, tetrominoes) = load_resources(config).unwrap();

        Self {
            game: Game::new(config, tetrominoes).unwrap(),
        }

        // for action in actions {
        //     if game.update(action)? == GameState::ToppedOut {
        //         break;
        //     }
        // }
    }

    fn get_move(&self) -> Action {
        let best = MySinglePlayerBot::tree_search(5, self.game.clone());

        if let Some((best_moves, _)) = best {
            println!("{:?}", best_moves);

            best_moves.last().unwrap_or(&Action::None).clone()
        } else {
            Action::None
        }
    }

    fn update_with(&mut self, action: Action) {
        self.game.update(action).expect("Failed to update.");
    }
}
