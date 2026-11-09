//! Parsing of action input files.

/// A player input on a frame
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    RotateClockwise,
    RotateCounterClockwise,
    MoveLeft,
    MoveRight,
    MoveDown,
    HardDrop,
    None,
}

/// TODO: give proper error type
/// turns input string into array of characters representing actions
pub fn parse_action_input(input: &str) -> Result<Vec<Action>, String> {
    input.chars().filter_map(|c| match c {
        'C' => Some(Ok(Action::RotateClockwise)),
        'W' => Some(Ok(Action::RotateCounterClockwise)),
        'L' => Some(Ok(Action::MoveLeft)),
        'R' => Some(Ok(Action::MoveRight)),
        'D' => Some(Ok(Action::MoveDown)),
        'H' => Some(Ok(Action::HardDrop)),
        'E' => Some(Ok(Action::None)),
        c => {
            if c.is_whitespace() {
                None
            } else {
                Some(Err(format!("TODO make me a real error (invalid action {c:?})")))
            }
        }
    }).collect()
}
