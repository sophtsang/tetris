use crate::resources::{Offset, Tetromino, Tetrominoes};

const BUFFER_SIZE: usize = 2;

pub struct Board {
    grid: Vec<u32>,
    pub width: usize,
    pub height: usize,
    tetrominoes: Tetrominoes,
    /// Current falling Tetromino, if one has been spawned.
    pub current_piece: Option<Tetromino>,
    /// Position of currently placed Tetromino.
    pub current_piece_position: Offset,
    /// Rotation index of currently placed Tetromino.
    pub current_piece_rotation_state_idx: usize,
}

impl Board {
    /// Create a new board.
    pub fn new(width: usize, height: usize, tetrominoes: Tetrominoes) -> Self {
        Self {
            // Grid includes buffer rows for showing a new piece.
            grid: vec![0; width * (height + BUFFER_SIZE)],
            width,
            height,
            tetrominoes,
            current_piece: None,
            current_piece_position: [(width as i32 - 1) / 2, height as i32],
            current_piece_rotation_state_idx: 0,
        }
    }

    #[cfg(test)]
    pub fn new_testing(
        width: usize,
        height: usize,
        tetrominoes: Tetrominoes,
        grid: Vec<u32>,
    ) -> Self {
        let mut board = Self::new(width, height, tetrominoes);
        assert_eq!(grid.len(), board.grid.len());
        board.grid = grid;
        board
    }

    /// Spawn a new tetromino if its starting cells do not overlap the board
    /// and there is not already a tetromino.
    pub fn spawn_piece(&mut self, piece: Tetromino) -> Result<(), String> {
        // Can't spawn a new piece when there is an existing piece on the board.
        if self.current_piece.is_some() {
            return Err("cannot spawn while another piece is active".to_string());
        }

        let position = [(self.width as i32 - 1) / 2, self.height as i32];

        // Check that the desired piece would not intersect with the grid when spawned.
        if self.overlaps(position, self.tetrominoes.config(piece).states[0]) {
            return Err("new piece overlaps the board at spawn".to_string());
        }

        // Spawn piece.
        self.current_piece = Some(piece);
        self.current_piece_position = position;
        self.current_piece_rotation_state_idx = 0;

        Ok(())
    }

    /// Index into the board grid given an offset (the board grid is flattened).
    pub fn index(&self, position: Offset) -> Result<usize, String> {
        // Check that position is positive.
        let [x, y] = position;
        if x < 0 || y < 0 {
            return Err(format!("board position {position:?} must be nonnegative"));
        }

        // Check that position is within the board.
        let (x, y) = (x as usize, y as usize);
        if x >= self.width || y >= self.height + BUFFER_SIZE {
            return Err(format!("board position {position:?} is out of bounds"));
        }

        Ok(y * self.width + x)
    }

    /// Check whether any cell of a tetromino hits a boundary or a locked cell.
    pub fn overlaps(&self, position: Offset, rotation: [Offset; 4]) -> bool {
        for [dx, dy] in rotation {
            let Ok(index) = self.index([position[0] + dx, position[1] + dy]) else {
                return true;
            };
            if self.grid[index] != 0 {
                return true;
            }
        }
        false
    }

    /// Get the 4 rotation offsets for `piece` in the config, relative to `rotation_state_idx`
    fn piece_rotation_offsets(&self, piece: Tetromino, rotation_state_idx: usize) -> [Offset; 4] {
        self.tetrominoes.config(piece).states[rotation_state_idx]
    }

    /// Return the active piece's cell offsets for its current rotation.
    pub fn current_rotation(&self) -> Result<[Offset; 4], String> {
        self.rotation_at(self.current_piece_rotation_state_idx)
    }

    fn rotation_at(&self, rotation_state_idx: usize) -> Result<[Offset; 4], String> {
        let piece = self.current_piece.ok_or("no current piece")?;
        // don't use piece_rotation_offsets, since that panics on wrong offset instead of giving a result
        // TODO: is that okay? not sure why previous writer cared about avoiding panic on this, should talk about
        self.tetrominoes
            .config(piece)
            .states
            .get(rotation_state_idx)
            .copied()
            .ok_or("invalid piece rotation state".to_string())
    }

    /// Lock a piece into the grid.
    pub fn lock_piece(&mut self) -> Result<(), String> {
        let rotation = self.current_rotation()?;

        // Check that current piece does not overlap already locked cells.
        if self.overlaps(self.current_piece_position, rotation) {
            return Err("current piece overlaps the board or is out of bounds".to_string());
        }

        // Lock in and clear piece.
        for [dx, dy] in rotation {
            let index = self.index([
                self.current_piece_position[0] + dx,
                self.current_piece_position[1] + dy,
            ])?;
            self.grid[index] = 1;
        }

        self.current_piece = None;

        Ok(())
    }

    /// Whether any locked cell occupies either buffer row.
    pub fn locked_cells_in_buffer(&self) -> bool {
        let first_buffer_cell = self.grid.len() - self.width * BUFFER_SIZE;
        self.grid[first_buffer_cell..].iter().any(|&cell| cell != 0)
    }

    /// Whether any cell of the current falling piece is in a buffer row.
    pub fn current_piece_in_buffer(&self) -> bool {
        let Ok(rotation) = self.current_rotation() else {
            return false;
        };
        rotation
            .into_iter()
            .any(|[_, dy]| self.current_piece_position[1] + dy >= self.height as i32)
    }

    /// Update the active piece if the proposed position and rotation fit.
    /// Returns an error without changing the piece when it would overlap.
    pub fn update_piece(
        &mut self,
        position: Offset,
        rotation_state_idx: usize,
    ) -> Result<(), String> {
        let rotation = self.rotation_at(rotation_state_idx)?;
        if self.overlaps(position, rotation) {
            return Err("updated piece would overlap the board or be out of bounds".to_string());
        }
        self.current_piece_position = position;
        self.current_piece_rotation_state_idx = rotation_state_idx;
        Ok(())
    }

    /// Remove full rows and shift all rows above them down.
    pub fn clear_full_rows(&mut self) -> usize {
        let mut next_row = 0;
        let row_count = self.height + BUFFER_SIZE;
        for row in 0..row_count {
            let start = row * self.width;
            let end = start + self.width;
            if self.grid[start..end].iter().all(|&cell| cell != 0) {
                continue;
            }
            self.grid.copy_within(start..end, next_row * self.width);
            next_row += 1;
        }
        self.grid[next_row * self.width..].fill(0);
        row_count - next_row
    }

    /// Serialize current board into a printable representation.
    pub fn serialize(&self, show_current_piece: bool) -> String {
        let mut cells = self.grid.clone();

        // Overlay the current piece onto the grid.
        if show_current_piece && let Ok(rotation) = self.current_rotation() {
            for [dx, dy] in rotation {
                if let Ok(index) = self.index([
                    self.current_piece_position[0] + dx,
                    self.current_piece_position[1] + dy,
                ]) {
                    cells[index] = 1;
                }
            }
        }

        // Collect the display grid into a string.
        (0..self.height + BUFFER_SIZE)
            .rev()
            .map(|y| {
                cells[y * self.width..(y + 1) * self.width]
                    .iter()
                    .map(|&cell| if cell == 0 { '.' } else { 'o' })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Checks if the current piece can move to [x+dx, y+dy] and returns this as an Offset if possible.
    /// If this is not a valid move, returns None.
    fn move_destination(&self, dx: i32, dy: i32) -> Option<Offset> {
        match self.current_piece {
            None => None, // there is no piece, so there is nothing to move
            Some(piece) => {
                let destination = [
                    self.current_piece_position[0] + dx,
                    self.current_piece_position[1] + dy,
                ];
                if self.overlaps(
                    destination,
                    self.piece_rotation_offsets(piece, self.current_piece_rotation_state_idx),
                ) {
                    None // destination position overlaps with something, cannot move there
                } else {
                    Some(destination)
                }
            }
        }
    }

    /// Whether there is free space for the current piece to go to [x+dx, y+dy].
    pub fn can_move(&self, dx: i32, dy: i32) -> bool {
        self.move_destination(dx, dy).is_some()
    }

    /// Tries to move the current piece to [x+dx, y+dy]. Returns whether the move succeeded.
    pub fn move_current_piece(&mut self, dx: i32, dy: i32) -> bool {
        match self.move_destination(dx, dy) {
            Some(destination) => {
                self.current_piece_position = destination;
                true
            }
            None => false,
        }
    }

    /// Attempts to rotate the current piece in the specified direction.
    /// Returns whether the rotation succeeded.
    pub fn rotate_current_piece(&mut self, clockwise: bool) -> bool {
        match self.current_piece {
            None => false,
            Some(piece) => {
                let config = self.tetrominoes.config(piece);
                let target_rotation = (self.current_piece_rotation_state_idx + if clockwise { 1 } else { 4 - 1 }) % 4; // +1 if clockwise, -1 if ccw (4-1 to avoid overflow)
                let target_rotation_states = config.states[target_rotation];
                let wallkick_table = [&config.wall_kick.o, &config.wall_kick.r, &config.wall_kick.t, &config.wall_kick.l];
                for i in 0..wallkick_table[self.current_piece_rotation_state_idx].len() {
                    let destination = [self.current_piece_position[0] + wallkick_table[self.current_piece_rotation_state_idx][i][0] - wallkick_table[target_rotation][i][0], self.current_piece_position[1] + wallkick_table[self.current_piece_rotation_state_idx][i][1] - wallkick_table[target_rotation][i][1]];
                    if !self.overlaps(destination, target_rotation_states) {
                        self.current_piece_position = destination;
                        self.current_piece_rotation_state_idx = target_rotation;
                        return true;
                    }
                }
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::TEST_TETROMINOES;

    fn expect_board(board: &Board, show_current_piece: bool, expected: &str) {
        let expected = expected
            .trim()
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n");
        let actual = board.serialize(show_current_piece);
        assert!(
            actual == expected,
            "board mismatch:\nexpected:\n{expected}\nactual:\n{actual}"
        );
    }

    #[test]
    fn spawning() {
        let mut board = Board::new(4, 2, TEST_TETROMINOES);
        expect_board(
            &board,
            true,
            r#"
            ....
            ....
            ....
            ....
        "#,
        );

        board.spawn_piece(Tetromino::O).expect("spawn O piece");

        expect_board(
            &board,
            true,
            r#"
            .oo.
            .oo.
            ....
            ....
        "#,
        );
    }

    #[test]
    fn locking() {
        let mut grid = vec![0; 4 * (2 + BUFFER_SIZE)];
        grid[0] = 1;
        let mut board = Board::new_testing(4, 2, TEST_TETROMINOES, grid);
        expect_board(
            &board,
            false,
            r#"
            ....
            ....
            ....
            o...
        "#,
        );

        board.spawn_piece(Tetromino::O).expect("spawn O piece");
        board.update_piece([2, 0], 0).expect("move O piece");
        expect_board(
            &board,
            true,
            r#"
            ....
            ....
            ..oo
            o.oo
        "#,
        );
        board.lock_piece().expect("lock O piece");

        assert_eq!(board.current_piece, None);
        expect_board(
            &board,
            true,
            r#"
            ....
            ....
            ..oo
            o.oo
        "#,
        );

        board.spawn_piece(Tetromino::O).expect("spawn next O piece");
        expect_board(
            &board,
            true,
            r#"
            .oo.
            .oo.
            ..oo
            o.oo
        "#,
        );
    }

    #[test]
    fn locking_on_previously_locked_cells_returns_an_error() {
        let mut board = Board::new(4, 2, TEST_TETROMINOES);
        board.spawn_piece(Tetromino::O).expect("spawn O piece");
        board.update_piece([1, 0], 0).expect("move O piece");
        board.lock_piece().expect("lock first O piece");
        expect_board(
            &board,
            false,
            r#"
            ....
            ....
            .oo.
            .oo.
        "#,
        );

        board.spawn_piece(Tetromino::O).expect("spawn next O piece");
        // Force an invalid position.
        board.current_piece_position = [1, 0];
        assert_eq!(
            board.lock_piece().unwrap_err(),
            "current piece overlaps the board or is out of bounds"
        );
        expect_board(
            &board,
            false,
            r#"
            ....
            ....
            .oo.
            .oo.
        "#,
        );
        assert_eq!(board.current_piece, Some(Tetromino::O));
    }

    #[test]
    fn spawning_on_a_locked_buffer_cell_returns_an_error() {
        let mut grid = vec![0; 4 * (2 + BUFFER_SIZE)];
        grid[2 * 4 + 1] = 1;
        let mut board = Board::new_testing(4, 2, TEST_TETROMINOES, grid);
        expect_board(
            &board,
            true,
            r#"
            ....
            .o..
            ....
            ....
        "#,
        );

        assert_eq!(
            board.spawn_piece(Tetromino::O).unwrap_err(),
            "new piece overlaps the board at spawn"
        );
        expect_board(
            &board,
            true,
            r#"
            ....
            .o..
            ....
            ....
        "#,
        );
        assert_eq!(board.current_piece, None);
    }

    #[test]
    fn clearing_a_full_row_moves_the_rows_above_down() {
        let mut grid = vec![0; 4 * (2 + BUFFER_SIZE)];
        grid[0..4].fill(1);
        grid[4] = 1;
        let mut board = Board::new_testing(4, 2, TEST_TETROMINOES, grid);

        expect_board(
            &board,
            false,
            r#"
            ....
            ....
            o...
            oooo
        "#,
        );
        assert_eq!(board.clear_full_rows(), 1);
        expect_board(
            &board,
            false,
            r#"
            ....
            ....
            ....
            o...
        "#,
        );
    }
}
