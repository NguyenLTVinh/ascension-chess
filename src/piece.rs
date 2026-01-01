use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PlayerColor,
    pub has_moved: bool,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: PlayerColor) -> Self {
        Self {
            piece_type,
            color,
            has_moved: false,
        }
    }

    pub fn value(&self) -> i32 {
        use crate::constants::*;
        match self.piece_type {
            PieceType::Pawn => VAL_PAWN,
            PieceType::Knight => VAL_KNIGHT,
            PieceType::Bishop => VAL_BISHOP,
            PieceType::Rook => VAL_ROOK,
            PieceType::Queen => VAL_QUEEN,
            PieceType::King => 0,
            PieceType::Hawk => VAL_HAWK,
            PieceType::Elephant => VAL_ELEPHANT,
            PieceType::Archbishop => VAL_ARCHBISHOP,
            PieceType::Cannon => VAL_CANNON,
            PieceType::Monarch => VAL_MONARCH,
        }
    }

    pub fn upgrade_cost(&self) -> Option<i32> {
        use crate::constants::*;
        match self.piece_type {
            PieceType::Pawn => Some(COST_HAWK),
            PieceType::Knight => Some(COST_ELEPHANT),
            PieceType::Bishop => Some(COST_ARCHBISHOP),
            PieceType::Rook => Some(COST_CANNON),
            PieceType::Queen => Some(COST_MONARCH),
            _ => None,
        }
    }

    pub fn upgraded_type(&self) -> Option<PieceType> {
        match self.piece_type {
            PieceType::Pawn => Some(PieceType::Hawk),
            PieceType::Knight => Some(PieceType::Elephant),
            PieceType::Bishop => Some(PieceType::Archbishop),
            PieceType::Rook => Some(PieceType::Cannon),
            PieceType::Queen => Some(PieceType::Monarch),
            _ => None,
        }
    }
}
