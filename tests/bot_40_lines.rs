use std::path::Path;

use p5_tetris::bot::{PlacementBot, SinglePlayerBot};
use p5_tetris::game::{Game, GameState};
use p5_tetris::load_resources::load_resources;

#[test]
fn bot_clears_40_lines() {
    const TARGET_LINES: usize = 40000;
    const MAX_FRAMES: usize = 20_000_000;

    let resources = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources");
    let (config, tetrominoes) = load_resources(&resources).unwrap();
    let mut game = Game::new(config, tetrominoes).unwrap();
    let mut bot = PlacementBot::new(&resources);
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
