use std::path::PathBuf;

use p5_tetris::bot::SinglePlayerBot;

pub type MySinglePlayerBot = p5_tetris::bot::PlacementBot;

#[test]
fn create_bot_and_get_any_action_test() {
    let config_path: PathBuf = "resources".into();
    let mut bot = MySinglePlayerBot::new(&config_path);
    let m = bot.get_move();
    bot.update_with(m);
}
