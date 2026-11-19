use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

pub mod action_inputs;
pub mod board;
pub mod bot;
pub mod game;
pub mod load_resources;
pub mod multiplayer;
pub mod random;
pub mod resources;

use action_inputs::{Action, parse_action_input};
use game::{Game, GameState};
use load_resources::load_resources;
use multiplayer::GameLog;
use resources::{Config, Tetrominoes};

/// Command-line arguments.
#[derive(Parser, Debug)]
struct Args {
    /// File of game inputs to simulate in single-player mode.
    #[arg(long, conflicts_with = "simulate_inputs_multi_player")]
    simulate_inputs_single_player: Option<PathBuf>,

    /// File of game inputs to simulate in multi-player mode.
    #[arg(long, conflicts_with = "simulate_inputs_single_player")]
    simulate_inputs_multi_player: Vec<PathBuf>,

    /// Directory containing config information:
    ///     config.json
    ///     wall_kicks.json
    ///     tetrominoes.json
    #[arg(long, default_value = "resources")]
    resources: PathBuf,

    /// Print the list of available bots
    #[arg(long)]
    list_bots: bool,
}

/// Prints the list of available bots.
fn list_bots() {
    // this is not kept in sync automatically with the actual bots we have
    // but tbh that would be overengineered for what we need
    let bots = ["good-bot", "bad-bot"];
    for bot in bots {
        println!("{bot}");
    }
}

fn run_single_player(
    path: &Path,
    config: Config,
    tetrominoes: Tetrominoes,
) -> Result<(), Box<dyn Error>> {
    let inputs =
        fs::read_to_string(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let actions = parse_action_input(&inputs)?; // dyn Error cuz who knows what error panic ? could be

    let mut game = Game::new(config, tetrominoes)?;
    for action in actions {
        if game.update(action)? == GameState::ToppedOut {
            break;
        }
    }
    println!("{}", game.serialize());
    Ok(())
}

/// Reads and parses one input file into a list of per-frame actions.
fn read_actions(path: &Path) -> Result<Vec<Action>, Box<dyn Error>> {
    let inputs =
        fs::read_to_string(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    Ok(parse_action_input(&inputs)?)
}

fn run_multi_player(
    paths: &[PathBuf],
    config: Config,
    tetrominoes: Tetrominoes,
) -> Result<(), Box<dyn Error>> {
    let mut game_log = GameLog::new(config, paths.len());
    let mut games = Vec::new();
    let mut actions = Vec::new();
    for path in paths {
        games.push(Game::new(config, tetrominoes.clone())?);
        actions.push(read_actions(path)?);
    }
    let frames = actions
        .iter()
        .map(|player_actions| player_actions.len())
        .max()
        .unwrap_or(0);

    for frame in 0..frames {
        if game_log
            .game_states
            .iter()
            .all(|&b| b == GameState::ToppedOut)
        {
            // are we all topped out
            break;
        }
        for player in 0..paths.len() {
            // games[idx].garbage().update(cleared: usize, opp_num_rows_to_create: usize, opp_cleared: usize) -> Option<{num_garbage: usize, empty_col: usize}>
            if let Some(action) = actions[player].get(frame)
                && game_log.game_states[player] != GameState::ToppedOut
            {
                match games[player].update(*action)? {
                    GameState::ToppedOut => game_log.topped_out(player), // kill the person
                    GameState::Playing => {}
                    GameState::Locked { rows_cleared } => {
                        game_log.update_cleared(player, rows_cleared)
                    }
                }
            }
        }
        game_log.update(&mut games)?;
    }

    // print both boards side by side, row by row, separated by four spaces
    let p1_board = games[0].serialize();
    let p2_board = games[1].serialize();
    for (p1_row, p2_row) in p1_board.lines().zip(p2_board.lines()) {
        println!("{p1_row}    {p2_row}");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let (config, tetrominoes) = load_resources(&args.resources)?;

    if args.list_bots {
        list_bots();
    } else if let Some(path) = &args.simulate_inputs_single_player {
        run_single_player(path, config, tetrominoes)?;
    } else if !args.simulate_inputs_multi_player.is_empty() {
        run_multi_player(&args.simulate_inputs_multi_player, config, tetrominoes)?;
    }
    Ok(())
}
