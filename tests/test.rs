use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use p5_tetris::action_inputs::Action;
use p5_tetris::bot::{SinglePlayerBot, TetrisAction};
use p5_tetris::game::{Game, GameState};
use p5_tetris::load_resources::load_resources;

pub struct MySinglePlayerBot {
    game: Game,
}

const HOLE_COST: f64 = 20.0;
const FLATNESS_COST: f64 = 1.0;
const HEIGHT_COST: f64 = 1.0;
const LINE_CLEAR_REWARD: f64 = 100.0;

impl MySinglePlayerBot {
    fn landing_reward(state: GameState, game: &Game) -> f64 {
        match state {
            GameState::Locked { rows_cleared } => {
                let board = &game.board;
                let board_cost = HOLE_COST * board.hole_count() as f64
                    + FLATNESS_COST * board.flatness_cost() as f64
                    + HEIGHT_COST * board.highest_filled_row().unwrap_or(0) as f64;
                (rows_cleared as f64 * LINE_CLEAR_REWARD) - board_cost
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
                let score = Self::landing_reward(state, &dropped);
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

    fn get_move(&self) -> TetrisAction {
        if let Some((mut path, _)) = self.best_drop() {
            path.push(Action::HardDrop);
            println!("{path:?}");
            path[0].into()
        } else {
            TetrisAction::E
        }
    }

    fn update_with(&mut self, action: TetrisAction) {
        self.game.update(action.into()).expect("Failed to update.");
    }
}

#[test]
fn create_bot_and_get_any_action_test() {
    let config_path: PathBuf = "resources".into();
    let mut bot = MySinglePlayerBot::new(&config_path);
    let m = bot.get_move();
    bot.update_with(m);
}

#[test]
fn bot_clears_40_lines() {
    const TARGET_LINES: usize = 40;
    const MAX_FRAMES: usize = 20_000;

    let resources = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    let (config, tetrominoes) = load_resources(&resources).unwrap();
    let mut game = Game::new(config, tetrominoes).unwrap();
    let mut bot = MySinglePlayerBot::new(&resources);
    let mut lines_cleared = 0;

    for frame in 1..=MAX_FRAMES {
        let action = bot.get_move();
        let state = game
            .update(action.into())
            .expect("game failed to process bot action");

        if let GameState::Locked { rows_cleared } = state {
            lines_cleared += rows_cleared;
        }
        println!(
            "Frame {frame}, action {action:?}, total rows cleared: {lines_cleared}:\n{}\n",
            game.serialize()
        );

        if state == GameState::ToppedOut {
            panic!(
                "bot topped out after {frame} frames with {lines_cleared}/{TARGET_LINES} lines cleared"
            );
        }

        if lines_cleared >= TARGET_LINES {
            return;
        }

        bot.update_with(action);
    }

    panic!("bot cleared {lines_cleared}/{TARGET_LINES} lines in {MAX_FRAMES} frames");
}
