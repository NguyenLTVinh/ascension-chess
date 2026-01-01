#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerColor {
    White,
    Black,
}

impl PlayerColor {
    pub fn opposite(&self) -> Self {
        match self {
            PlayerColor::White => PlayerColor::Black,
            PlayerColor::Black => PlayerColor::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
    Hawk,
    Elephant,
    Archbishop,
    Cannon,
    Chancellor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn is_valid(&self) -> bool {
        self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8
    }
}
