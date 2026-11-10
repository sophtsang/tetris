//! Contains our resource and config types.

use std::collections::HashMap;
use std::error::Error;

use serde::Deserialize;

/// Game config, parsed from `config.json`.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Dimensions of game board.
    pub width: u32,
    pub height: u32,
    /// Rate at which piece drops on board.
    pub gravity: u32,
    pub gravity_decrease: u32,
    /// Once this many frames pass without dropping, lock the piece.
    pub lock_delay: u32,
    /// Random seed for next piece generation.
    pub random_seed: [u8; 16],
}

/// (x, y) offset.
pub type Offset = [i32; 2];

/// Wall kick offsets to try when rotating.
#[derive(Deserialize, Debug, Clone)]
pub struct Wallkick {
    /// 0 degrees
    pub o: Vec<Offset>,
    /// 90 degrees (clockwise)
    pub r: Vec<Offset>,
    /// 180 degrees
    pub t: Vec<Offset>,
    /// 270 degrees (clockwise)
    pub l: Vec<Offset>,
}

/// A tetromino's rotation states and its wall kicks config.
#[derive(Debug)]
pub struct TetrominoConfig {
    pub states: [[Offset; 4]; 4],
    pub wall_kick: Wallkick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tetromino {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl Tetromino {
    pub const ALL: [Tetromino; 7] = [
        Tetromino::I,
        Tetromino::J,
        Tetromino::L,
        Tetromino::O,
        Tetromino::S,
        Tetromino::T,
        Tetromino::Z,
    ];
}

/// Stores config for each tetromino type.
#[derive(Debug)]
pub struct Tetrominoes {
    i: TetrominoConfig,
    j: TetrominoConfig,
    l: TetrominoConfig,
    o: TetrominoConfig,
    s: TetrominoConfig,
    t: TetrominoConfig,
    z: TetrominoConfig,
}

impl Tetrominoes {
    /// Builds Tetrominoes config lookup from discovered configs map.
    pub fn new(mut configs: HashMap<String, TetrominoConfig>) -> Result<Self, Box<dyn Error>> {
        let mut take = |t: &str| -> Result<TetrominoConfig, Box<dyn Error>> {
            configs
                .remove(t)
                .ok_or_else(|| format!("tetrominoes.json is missing \"{t}\"").into())
        };
        Ok(Self {
            i: take("i")?,
            j: take("j")?,
            l: take("l")?,
            o: take("o")?,
            s: take("s")?,
            t: take("t")?,
            z: take("z")?,
        })
    }

    pub fn config(&self, tetromino: Tetromino) -> &TetrominoConfig {
        match tetromino {
            Tetromino::I => &self.i,
            Tetromino::J => &self.j,
            Tetromino::L => &self.l,
            Tetromino::O => &self.o,
            Tetromino::S => &self.s,
            Tetromino::T => &self.t,
            Tetromino::Z => &self.z,
        }
    }
}
