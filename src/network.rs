use crate::types::{PieceType, PlayerColor, Pos};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameMessage {
    Join { room: String },
    Welcome { color: PlayerColor },
    Move { from: Pos, to: Pos },
    Upgrade { pos: Pos },
    Promote { piece_type: PieceType },
    Error { message: String },
    OpponentDisconnected,
}
