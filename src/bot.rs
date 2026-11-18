use crate::action_inputs::Action;
use crate::game::{Game, GameState};
use crate::load_resources::load_resources;
use crate::multiplayer::GameLog;
use rayon::prelude::*;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
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

pub struct PlacementBot {
    game: Game,
    plan: RefCell<VecDeque<Action>>,
}

pub struct MultiplayerPlacementBot {
    games: [Game; 2],
    log: GameLog,
    plan: RefCell<VecDeque<Action>>,
}

const HOLE_COST: f64 = 20.0;
const FLATNESS_COST: f64 = 1.0;
const HEIGHT_COST: f64 = 1.0;
const LINE_CLEAR_REWARD: f64 = 200.0;
/// Number of generated pieces to consider after the current piece.
/// DEPTH = 2 plans placements for the current piece and the next two.
pub const DEPTH: usize = 2;
const TOP_OUT_COST: f64 = 1_000_000.0;

impl PlacementBot {
    fn board_cost(game: &Game) -> f64 {
        let board = &game.board;
        HOLE_COST * board.hole_count() as f64
            + FLATNESS_COST * board.flatness_cost() as f64
            + HEIGHT_COST * board.highest_filled_row().unwrap_or(0) as f64
    }

    /// Enumerate legal setup positions at the current height, then simulate a
    /// hard drop from each one. The game clone reveals the next generated piece.
    fn drop_options(game: &Game) -> Vec<(Vec<Action>, GameState, Game)> {
        let start = game.board.clone();
        if start.current_piece.is_none() {
            return Vec::new();
        }
        let start_y = start.current_piece_position[1];
        let mut queue = VecDeque::from([(start.clone(), Vec::<Action>::new())]);
        let mut visited = HashSet::from([(
            start.current_piece_position[0],
            start.current_piece_rotation_state_idx,
        )]);
        let mut options = Vec::new();

        while let Some((board, path)) = queue.pop_front() {
            let mut dropped = game.clone();
            dropped.board = board.clone();
            if let Ok(state) = dropped.update(Action::HardDrop) {
                options.push((path.clone(), state, dropped));
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

        options
    }

    /// Score exactly `pieces_left` future placements. Board costs are applied
    /// once at the leaf, while line clears from every placement are rewarded.
    fn future_score(game: &Game, pieces_left: usize) -> f64 {
        if pieces_left == 0 {
            return -Self::board_cost(game);
        }

        Self::drop_options(game)
            .into_iter()
            .map(|(_, state, next_game)| Self::score_drop(state, &next_game, pieces_left - 1))
            .reduce(f64::max)
            .unwrap_or(-TOP_OUT_COST)
    }

    fn score_drop(state: GameState, game: &Game, future_pieces: usize) -> f64 {
        match state {
            GameState::Locked { rows_cleared } => {
                rows_cleared as f64 * LINE_CLEAR_REWARD + Self::future_score(game, future_pieces)
            }
            GameState::ToppedOut => -TOP_OUT_COST,
            GameState::Playing => unreachable!("hard drop always locks the piece"),
        }
    }

    fn best_drop(game: &Game) -> Option<(Vec<Action>, f64)> {
        Self::drop_options(game)
            .into_par_iter()
            .enumerate()
            .map(|(index, (path, state, next_game))| {
                let score = Self::score_drop(state, &next_game, DEPTH);
                (index, path, score)
            })
            .max_by(|(index_a, path_a, score_a), (index_b, path_b, score_b)| {
                score_a
                    .total_cmp(score_b)
                    .then_with(|| path_b.len().cmp(&path_a.len()))
                    .then_with(|| index_a.cmp(index_b))
            })
            .map(|(_, path, score)| (path, score))
    }

    fn move_for(game: &Game, plan: &RefCell<VecDeque<Action>>, print_plan: bool) -> TetrisAction {
        let mut plan = plan.borrow_mut();
        if plan.is_empty()
            && let Some((mut path, _)) = Self::best_drop(game)
        {
            path.push(Action::HardDrop);
            if print_plan {
                println!("{path:?}");
            }
            *plan = path.into();
        }
        plan.front().copied().unwrap_or(Action::None).into()
    }

    fn advance_plan(plan: &RefCell<VecDeque<Action>>, action: TetrisAction, still_valid: bool) {
        let mut plan = plan.borrow_mut();
        if plan.front().copied() == Some(action.into()) && still_valid {
            plan.pop_front();
        } else {
            plan.clear();
        }
    }
}

impl SinglePlayerBot for PlacementBot {
    fn new(config: &Path) -> Self {
        let (config, tetrominoes) = load_resources(config).unwrap();
        Self {
            game: Game::new(config, tetrominoes).unwrap(),
            plan: RefCell::new(VecDeque::new()),
        }
    }

    fn get_move(&self) -> TetrisAction {
        Self::move_for(&self.game, &self.plan, true)
    }

    fn update_with(&mut self, action: TetrisAction) {
        let previous_y = self.game.board.current_piece_position[1];
        let state = self.game.update(action.into()).expect("Failed to update.");
        let still_valid =
            state == GameState::Playing && self.game.board.current_piece_position[1] == previous_y;
        Self::advance_plan(&self.plan, action, still_valid);
    }
}

impl MultiPlayerBot for MultiplayerPlacementBot {
    fn new(config: &Path) -> Self {
        let (config, tetrominoes) = load_resources(config).unwrap();
        let games = [
            Game::new(config, tetrominoes.clone()).unwrap(),
            Game::new(config, tetrominoes).unwrap(),
        ];
        Self {
            games,
            log: GameLog::new(config, 2),
            plan: RefCell::new(VecDeque::new()),
        }
    }

    fn get_move(&self) -> TetrisAction {
        if self.log.game_states[0] == GameState::ToppedOut {
            TetrisAction::E
        } else {
            PlacementBot::move_for(&self.games[0], &self.plan, false)
        }
    }

    fn update_with(&mut self, p_actions: &[TetrisAction]) {
        assert_eq!(p_actions.len(), 2, "expected one action per player");
        let previous_y = self.games[0].board.current_piece_position[1];
        let mut own_state = None;

        for (player, action) in p_actions.iter().enumerate() {
            if self.log.game_states[player] == GameState::ToppedOut {
                continue;
            }
            let state = self.games[player]
                .update((*action).into())
                .expect("Failed to update.");
            if player == 0 {
                own_state = Some(state);
            }
            match state {
                GameState::Playing => {}
                GameState::ToppedOut => self.log.topped_out(player),
                GameState::Locked { rows_cleared } => self.log.update_cleared(player, rows_cleared),
            }
        }
        self.log
            .update(&mut self.games)
            .expect("Failed to update garbage.");
        let still_valid = own_state == Some(GameState::Playing)
            && self.log.game_states[0] != GameState::ToppedOut
            && self.games[0].board.current_piece_position[1] == previous_y;
        PlacementBot::advance_plan(&self.plan, p_actions[0], still_valid);
    }
}
