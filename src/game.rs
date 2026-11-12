use crate::action_inputs::Action;
use crate::board::Board;
use crate::random::PieceGenerator;
use crate::resources::{Config, Tetromino, Tetrominoes};

#[derive(PartialEq)]
pub enum GameState {
    Playing,
    ToppedOut,
}

pub struct Game {
    board: Board,
    config: Config,
    piece_generator: PieceGenerator,
    // a way to represent the number of frames before the piece moves
    // down a cell
    frames_since_last_gravity_event: u32,
    rotations_since_moved_down: u32,
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
    pub fn new(config: Config, tetrominoes: Tetrominoes) -> Result<Self, String> {
        let piece_generator = PieceGenerator::new(Tetromino::ALL, config.random_seed);
        let tetromino = *piece_generator.cur();
        let mut board = Board::new(config.width as usize, config.height as usize, tetrominoes);
        board.spawn_piece(tetromino)?;
        Ok(Self {
            board,
            config: config,
            piece_generator: piece_generator,
            frames_since_last_gravity_event: 0,
            rotations_since_moved_down: 0,
        })
    }

    /// Show the board with its current falling piece.
    pub fn serialize(&self) -> String {
        self.board.serialize(true)
    }

    // called when a new piece is spawned
    fn spawn_new_piece(&mut self) -> Result<(), String> {
        self.piece_generator.next();
        self.board.spawn_piece(*self.piece_generator.cur())?;
        self.rotations_since_moved_down = 0;
        self.frames_since_last_gravity_event = 0;
        Ok(())
    }

    fn lock_current_piece(&mut self) -> Result<GameState, String> {
        // if LOCKED: as in a piece cannot move down anymore or be rotated
        // 0. draw piece on board ->
        // 1. check if topped out? ->
        // 2. can clear full rows? ->
        // 3. generate a new piece
        let piece_was_in_buffer = self.board.current_piece_in_buffer();
        self.board.lock_piece()?;
        if piece_was_in_buffer || self.board.locked_cells_in_buffer() {
            Ok(GameState::ToppedOut) // return early when lose
        } else {
            self.board.clear_full_rows();
            if self.spawn_new_piece().is_err() {
                Ok(GameState::ToppedOut)
            } else {
                Ok(GameState::Playing)
            }
        }
    }

    // On every on_tick or frame update, Game updates.
    pub fn update(&mut self, action: Action) -> Result<GameState, String> {
        match action {
            Action::None | Action::MoveDown => {}
            Action::RotateClockwise => {
                if self.board.rotate_current_piece(true) {
                    self.rotations_since_moved_down += 1
                }
            }
            Action::RotateCounterClockwise => {
                if self.board.rotate_current_piece(false) {
                    self.rotations_since_moved_down += 1
                }
            }
            Action::MoveLeft => {
                self.board.move_current_piece(-1, 0);
            }
            Action::MoveRight => {
                self.board.move_current_piece(1, 0);
            }
            Action::HardDrop => {
                while self.board.move_current_piece(0, -1) {}
                return self.lock_current_piece();
            }
        }

        let gravity = match action {
            Action::MoveDown => self
                .config
                .gravity
                .saturating_sub(self.config.gravity_decrease), // soft drop
            _ => self.config.gravity, // the amt of frames that occur before dropping 1 cell
        };

        self.frames_since_last_gravity_event += 1;
        let can_move_down = self.board.can_move(0, -1);

        let locked =
            self.frames_since_last_gravity_event >= self.config.lock_delay && !can_move_down;

        if locked {
            return self.lock_current_piece();
        } else if can_move_down && self.frames_since_last_gravity_event >= gravity {
            // moving current piece is guaranteed to be possible, so no need to check what this returns
            self.board.move_current_piece(0, -1);
            self.frames_since_last_gravity_event = 0;
            self.rotations_since_moved_down = 0;
        };

        Ok(GameState::Playing)
    }
}
