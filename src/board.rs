use crate::resources::{Tetromino, Tetrominoes, Offset, Config};

pub struct Board {
    grid: Vec<Vec<u32>>
}

impl Board {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            grid: vec![vec![0; width as usize]; height as usize]
        }
    }

    pub fn update(&mut self, position: Offset, rotation: [Offset; 4]) {
        todo!("update grid somehow")
    }

    pub fn topped_out(&self) -> bool {
        todo!("check whether any cells in rows 20-21, occupied")
    }

    pub fn can_move_down(&self, position: Offset, rotation: [Offset; 4]) -> bool {
        todo!("true is can move down vertically")
    }

    pub fn serialize(&self) -> String {
        todo!("serialize the grid to string rep")
    }
}