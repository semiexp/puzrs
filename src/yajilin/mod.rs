mod field;
mod generator;

pub use self::field::*;
pub use self::generator::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clue {
    NoClue,
    Empty,
    Up(i32),
    Left(i32),
    Down(i32),
    Right(i32),
}

impl Clue {
    pub fn same_shape(self, other: Clue) -> bool {
        match (self, other) {
            (Clue::Up(_), Clue::Up(_))
            | (Clue::Left(_), Clue::Left(_))
            | (Clue::Down(_), Clue::Down(_))
            | (Clue::Right(_), Clue::Right(_)) => true,
            _ => false,
        }
    }
    pub fn clue_number(self) -> i32 {
        match self {
            Clue::NoClue | Clue::Empty => -1,
            Clue::Up(n) | Clue::Left(n) | Clue::Down(n) | Clue::Right(n) => n,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Clue,
    Undecided,
    Line,
    Blocked,
}

impl Cell {
    pub fn is_blocking(self) -> bool {
        match self {
            Cell::Clue | Cell::Blocked => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Technique {
    pub two_by_two: bool,
    pub two_by_three: bool,
    pub one_in_three_orthogonal_either: bool,
    pub one_in_three_remote: bool,
    pub inout_advanced: bool,
    pub local_parity: bool,
}

impl Technique {
    pub fn new() -> Technique {
        Technique {
            two_by_two: true,
            two_by_three: true,
            one_in_three_orthogonal_either: true,
            one_in_three_remote: true,
            inout_advanced: true,
            local_parity: true,
        }
    }
}
