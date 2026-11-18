use std::path::PathBuf;

use p5_tetris::bot::{MultiPlayerBot, SinglePlayerBot, TetrisAction};

pub type MySinglePlayerBot = p5_tetris::bot::PlacementBot;
pub type MyMultiPlayerBot = p5_tetris::bot::MultiplayerPlacementBot;

#[test]
fn create_bot_and_get_any_action_test() {
    let config_path: PathBuf = "resources".into();
    let mut bot = MySinglePlayerBot::new(&config_path);
    let m = bot.get_move();
    bot.update_with(m);
}

#[test]
fn create_mut_bot_and_get_any_action_test() {
    let config_path: PathBuf = "resources".into();
    let mut bot = MyMultiPlayerBot::new(&config_path);
    let m = bot.get_move();
    bot.update_with(&[m, TetrisAction::E]);
}
