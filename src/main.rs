use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;

mod action_inputs;
mod load_resources;
mod resources;

use action_inputs::parse_action_input;
use load_resources::load_resources;
use resources::Tetromino;

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

    // Debug output to show parsing worked; remove once the game is wired up.
    println!("{args:#?}");
    println!("{config:#?}");
    for t in [Tetromino::I, Tetromino::O, Tetromino::T] {
        println!("{t:?}: {:?}", tetrominoes.config(t));
    }

    if let Some(path) = &args.simulate_inputs_single_player {
        let inputs = fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        println!("inputs from {}:", path.display());
        let actions = parse_action_input(&inputs)?;
        for (frame, action) in actions.iter().enumerate() {
            println!("  frame {frame}: {action:?}");
        }
    }

    // TODO: make game, run game loop
    // let piece_generator = PieceGenerator::new(bag, seed);
    // let board = Board::new();
    // let game = Game {
    //      board: board, 
    //      piece_generator: piece_generator,
    //      current_piece: piece_generator.cur(),

    // IN THE LOOP:
    //      check if frame == 0 then generate piece
    //      move the piece
    // cam: actions: rotations and wall_kick
    //       figuring out when a piece has finished falling -> turn into cells
    // 
    // me: i will increment frame, check when to generate new piece, 
    //      
    //
    // sid: serialize the board
    // }

    Ok(())
}
