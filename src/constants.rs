use macroquad::prelude::*;

pub const BOARD_SIZE: usize = 8;
pub const SQUARE_SIZE: f32 = 80.0;
pub const BOARD_OFFSET_X: f32 = 50.0;
pub const BOARD_OFFSET_Y: f32 = 50.0;

pub const COST_HAWK: i32 = 5;
pub const COST_ELEPHANT: i32 = 7;
pub const COST_ARCHBISHOP: i32 = 7;
pub const COST_CANNON: i32 = 8;
pub const COST_CHANCELLOR: i32 = 12;

pub const VAL_PAWN: i32 = 1;
pub const VAL_KNIGHT: i32 = 3;
pub const VAL_BISHOP: i32 = 3;
pub const VAL_ROOK: i32 = 5;
pub const VAL_QUEEN: i32 = 9;
pub const VAL_HAWK: i32 = 6;
pub const VAL_ELEPHANT: i32 = 10;
pub const VAL_ARCHBISHOP: i32 = 10;
pub const VAL_CANNON: i32 = 13;
pub const VAL_CHANCELLOR: i32 = 21;

pub const SELECTION_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.5);
pub const MOVE_HINT_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.5);
pub const CAPTURE_HINT_COLOR: Color = Color::new(0.0, 1.0, 0.0, 0.5);
