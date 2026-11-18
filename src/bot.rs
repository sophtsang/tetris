use crate::action_inputs::Action;
use std::path::Path;

/// Input symbols required by the bot interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrisAction {
    C,
    W,
    L,
    R,
    D,
    H,
    E,
}

impl From<TetrisAction> for Action {
    fn from(action: TetrisAction) -> Self {
        match action {
            TetrisAction::C => Action::RotateClockwise,
            TetrisAction::W => Action::RotateCounterClockwise,
            TetrisAction::L => Action::MoveLeft,
            TetrisAction::R => Action::MoveRight,
            TetrisAction::D => Action::MoveDown,
            TetrisAction::H => Action::HardDrop,
            TetrisAction::E => Action::None,
        }
    }
}

impl From<Action> for TetrisAction {
    fn from(action: Action) -> Self {
        match action {
            Action::RotateClockwise => TetrisAction::C,
            Action::RotateCounterClockwise => TetrisAction::W,
            Action::MoveLeft => TetrisAction::L,
            Action::MoveRight => TetrisAction::R,
            Action::MoveDown => TetrisAction::D,
            Action::HardDrop => TetrisAction::H,
            Action::None => TetrisAction::E,
        }
    }
}

pub trait SinglePlayerBot {
    /// Create a new bot given the path to the resources dir.
    fn new(config: &Path) -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(&self) -> TetrisAction;

    /// Updates the current bot state assuming the player plays a given action.
    fn update_with(&mut self, action: TetrisAction);
}

pub trait MultiPlayerBot {
    /// Create a new bot given the path to the resources dir.
    fn new(config: &Path) -> Self;

    /// Gets an action to perform based on the board state. This should be called
    /// every frame.
    fn get_move(&self) -> TetrisAction;

    /// Updates the bot after every player's action in a frame.
    fn update_with(&mut self, p_actions: &[TetrisAction]);
}
