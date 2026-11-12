use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;

mod action_inputs;
mod board;
mod game;
mod load_resources;
mod random;
mod resources;

use action_inputs::parse_action_input;
use game::{Game, GameState};
use load_resources::load_resources;

/// Command-line arguments.
#[derive(Parser, Debug)]
struct Args {
    /// File of game inputs to simulate in single-player mode.
    #[arg(long)]
    simulate_inputs_single_player: Option<PathBuf>,

    /// Directory containing config information:
    ///     config.json
    ///     wall_kicks.json
    ///     tetrominoes.json
    #[arg(long, default_value = "resources")]
    resources: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (config, tetrominoes) = load_resources(&args.resources)?;

    if let Some(path) = &args.simulate_inputs_single_player {
        let inputs = fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        let actions = parse_action_input(&inputs)?;

        let mut game = Game::new(config, tetrominoes)?;
        for action in actions {
            if game.update(action)? == GameState::ToppedOut {
                break;
            }
        }
        println!("{}", game.serialize());
    }
    Ok(())
}
