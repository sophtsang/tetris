use crate::action_inputs::Action;
use crate::game::{Game, GameState};
use crate::load_resources::load_resources;

use std::path::Path;

pub trait SinglePlayerBot {
    /// Create a new bot given the path to the resources dir.
    fn new(config: &Path) -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(&mut self) -> Action;

    /// Updates the current bot state assuming the player plays a given action.
    /// If the bot is playing, this will likely be the result of
    /// `bot.get_move()`.
    fn update_with(&mut self, action: Action);
}

pub struct MySinglePlayerBot {
    game: Game,
    best_seq: Option<Vec<Action>>,
}

impl MySinglePlayerBot {
    fn reward(game_state: &GameState, before_bottom_row: i32, game_after_action: &Game) -> i32 {
        match game_state {
            GameState::Locked { rows_cleared } => {
                (*rows_cleared as i32) * 100 + before_bottom_row
                    - game_after_action
                        .last_locked_bottom_row()
                        .expect("locked game must record the piece's bottom row")
            }
            GameState::Playing => {
                before_bottom_row
                    - game_after_action
                        .distance_to_board_bottom()
                        .expect("playing game must have an active piece")
            }
            GameState::ToppedOut => -1000,
        }
    }

    fn tree_search(remaining_layers: u32, game: Game) -> Option<(Vec<Action>, i32)> {
        let mut best_seq = None;
        let before_bottom_row = game
            .distance_to_board_bottom()
            .expect("tree search requires an active piece");

        for action in [
            Action::MoveLeft,
            Action::MoveRight,
            Action::RotateClockwise,
            Action::RotateCounterClockwise,
            Action::MoveDown,
            Action::HardDrop,
            Action::None,
        ] {
            let mut game_after_action = game.clone();

            if let Ok(game_state) = game_after_action.update(action) {
                let move_reward =
                    MySinglePlayerBot::reward(&game_state, before_bottom_row, &game_after_action);

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
            best_seq: None,
        }

        // for action in actions {
        //     if game.update(action)? == GameState::ToppedOut {
        //         break;
        //     }
        // }
    }

    fn get_move(&mut self) -> Action {
        if self.best_seq.as_ref().is_none_or(Vec::is_empty) {
            let best = MySinglePlayerBot::tree_search(6, self.game.clone());

            if let Some((best_moves, _)) = best {
                self.best_seq = Some(best_moves);
            }
        }

        self.best_seq
            .as_mut()
            .and_then(Vec::pop)
            .unwrap_or(Action::None)
    }

    fn update_with(&mut self, action: Action) {
        self.game.update(action).expect("Failed to update.");
    }
}
