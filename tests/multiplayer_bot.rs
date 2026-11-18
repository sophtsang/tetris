use std::path::PathBuf;

use p5_tetris::bot::{MultiPlayerBot, TetrisAction};

pub type MyMultiPlayerBot = p5_tetris::bot::MultiplayerPlacementBot;

#[test]
fn create_multiplayer_bot_and_get_any_action_test() {
    let config_path: PathBuf = "resources".into();
    let mut bot = MyMultiPlayerBot::new(&config_path);
    let action = bot.get_move();
    bot.update_with(&[action, TetrisAction::E]);
}
