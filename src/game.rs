use crate::board::Board;
use crate::random::PieceGenerator;
use crate::resources::{Tetromino, Tetrominoes, Offset, Config};
use crate::action_inputs::Action;

#[derive(PartialEq)]
pub enum GameState {
    Playing,
    ToppedOut
}

pub struct Game {
    board: Board,
    config: Config,
    tetrominoes: Tetrominoes,
    piece_generator: PieceGenerator,
    current_piece: Tetromino,
    current_piece_position: Offset,
    // the 4 cells that the piece occupies if centered at (0, 0) position
    // TBH current_piece_rotation is not necessary
    // current_piece_rotation: [Offset; 4],
    // this stores the index of the piece's state element in corresponding
    // tetrominoes.json "states"
    current_piece_state_idx: usize, 
    // a way to represent the number of frames before the piece moves
    // down a cell
    frames_since_last_gravity_event: u32, 
    rotations_since_moved_down: u32
}

// pub struct Config {
//     /// Dimensions of game board.
//     pub width: u32,
//     pub height: u32,
//     /// Rate at which piece drops on board.
//     pub gravity: u32,
//     pub gravity_decrease: u32,
//     /// Once this many frames pass without dropping, lock the piece.
//     pub lock_delay: u32,
//     /// Random seed for next piece generation.
//     pub random_seed: [u8; 16],
// }
impl Game {
    pub fn new(config: Config, tetrominoes: Tetrominoes) -> Self {
        let piece_generator = PieceGenerator::new(Tetromino::ALL, config.random_seed);
        let tetromino = *piece_generator.cur();
        let tetromino_config = tetrominoes.config(tetromino);
        let position = [(config.width as i32 - 1) / 2, config.height as i32];
        Self {
            board: Board::new(config.width, config.height),
            config: config,
            tetrominoes: tetrominoes,
            piece_generator: piece_generator,
            current_piece: tetromino,
            current_piece_position: position, // middle top
            current_piece_state_idx: 0,
            frames_since_last_gravity_event: 0,
            rotations_since_moved_down: 0
        }
    }

    // called when a new piece is spawned
    fn spawn_new_piece(&mut self) {
        self.piece_generator.next();
        // new piece set most stuff to 0
        self.current_piece = *self.piece_generator.cur();
        self.current_piece_position = [(self.config.width as i32 - 1) / 2, self.config.height as i32];
        self.current_piece_state_idx = 0;
        self.rotations_since_moved_down = 0;
        self.frames_since_last_gravity_event = 0;
    }

    fn get_piece_rotation(&self) -> [Offset; 4] {
         self.tetrominoes.config(self.current_piece).states[self.current_piece_state_idx]
    }

    // On every on_tick or frame update, Game updates.
    pub fn update(&mut self, action: Action) -> GameState {
        // should this be a large match action ? idk maybe
        // also no idea what wall_kick is so that is taken care of
        // somewhere here

        // ROTATIONS STUFF: match action maybe?
        todo!("apply the action on tetromino, 
               if action was CW -> increment or CCW -> wall_kick + maybe change piece_state_idx
               to get new tetromino_config.states[idx] = current_piece_rotation");

        todo!("if rotation successful, rotations_since_moved_down += 1");

        todo!("if the action was L or R, modify current_piece_position.x");

        let gravity = match action {
            Action::MoveDown => self.config.gravity.saturating_sub(self.config.gravity_decrease), // soft drop
            _ => self.config.gravity // the amt of frames that occur before dropping 1 cell
        };
        
        self.frames_since_last_gravity_event += 1;
        let can_move_down = self.board.can_move_down(
            self.current_piece_position, // pose, rotation
            self.get_piece_rotation()
        ); // specifically if there are vertical collision

        let locked = if action == Action::HardDrop {
            todo!("if action is HardDrop, directly set current_piece_position.y to the floor 
                (closest vertical occupied cell) and then immediately lock");
            true
        } else {
            self.frames_since_last_gravity_event > self.config.lock_delay && !can_move_down
        };

        if locked { 
            // if LOCKED: as in a piece cannot move down anymore or be rotated
            // 0. draw piece on board ->
            // 1. check if topped out? -> 
            // 2. can clear full rows? -> 
            // 3. generate a new piece
            self.board.update(
                self.current_piece_position, // pose, rotation
                self.get_piece_rotation()
            );
            if self.board.topped_out() {
                todo!("check if topped out after position and frame update, 
                    probably done via board::check if rows 20-21 are occupied,
                    if yes, die");

                return GameState::ToppedOut; // return early when lose
            } else {
                todo!("board checks if there are full rows: board::clear_full_rows()");

                self.spawn_new_piece();
            }
        } else if can_move_down && self.frames_since_last_gravity_event > gravity {
            // guaranteed that move down 1 cell, so no vertical collisions
            self.current_piece_position[1] -= 1;
            self.frames_since_last_gravity_event = 0;
            self.rotations_since_moved_down = 0;
        };

        GameState::Playing
    }
}