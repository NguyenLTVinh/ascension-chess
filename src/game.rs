use crate::board::*;
use crate::types::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TurnPhase {
    Normal,
    PostUpgrade(Pos),
    Promoting(Pos, bool),
    GameOver,
}

pub struct Game {
    pub board: Board,
    pub turn: PlayerColor,
    pub white_points: i32,
    pub black_points: i32,
    pub selected_pos: Option<Pos>,
    pub legal_moves: Vec<Pos>,
    pub phase: TurnPhase,
    pub winner: Option<PlayerColor>,
    pub history: Vec<Board>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            turn: PlayerColor::White,
            white_points: 0,
            black_points: 0,
            selected_pos: None,
            legal_moves: Vec::new(),
            phase: TurnPhase::Normal,
            winner: None,
            history: Vec::new(),
        }
    }

    pub fn start_turn(&mut self) {
        match self.turn {
            PlayerColor::White => self.white_points += 1,
            PlayerColor::Black => self.black_points += 1,
        }
        self.phase = TurnPhase::Normal;
    }

    pub fn select_square(&mut self, pos: Pos) {
        if self.phase == TurnPhase::GameOver {
            return;
        }
        if let TurnPhase::Promoting(_, _) = self.phase {
            return;
        }

        if Some(pos) == self.selected_pos {
            self.selected_pos = None;
            self.legal_moves.clear();
            return;
        }

        if self.legal_moves.contains(&pos) {
            if let Some(from) = self.selected_pos {
                self.make_move(from, pos);
                return;
            }
        }

        if let Some(piece) = self.board.get_piece(pos) {
            if piece.color == self.turn {
                if let TurnPhase::PostUpgrade(upgraded_pos) = self.phase {
                    if pos == upgraded_pos {
                        return;
                    }
                }

                self.selected_pos = Some(pos);
                self.legal_moves = self.board.get_legal_moves(pos);
            } else {
                self.selected_pos = None;
                self.legal_moves.clear();
            }
        } else {
            self.selected_pos = None;
            self.legal_moves.clear();
        }
    }

    pub fn make_move(&mut self, from: Pos, to: Pos) {
        let piece = self.board.get_piece(from).unwrap();
        let target = self.board.get_piece(to);

        let mut points_gained = 0;
        if let Some(captured) = target {
            points_gained += captured.value();
            if captured.piece_type == PieceType::King {
                self.winner = Some(self.turn);
                self.phase = TurnPhase::GameOver;
                self.board.set_piece(to, Some(piece));
                self.board.set_piece(from, None);
                return;
            }
        }

        let mut moved_piece = piece;
        moved_piece.has_moved = true;
        self.board.set_piece(to, Some(moved_piece));
        self.board.set_piece(from, None);

        if piece.piece_type == PieceType::King && (from.x - to.x).abs() == 2 {
            let rook_x = if to.x > from.x { 7 } else { 0 };
            let rook_dest_x = if to.x > from.x { to.x - 1 } else { to.x + 1 };
            let rook_pos = Pos::new(rook_x, from.y);
            let rook_dest = Pos::new(rook_dest_x, from.y);
            if let Some(mut rook) = self.board.get_piece(rook_pos) {
                rook.has_moved = true;
                self.board.set_piece(rook_dest, Some(rook));
                self.board.set_piece(rook_pos, None);
            }
            points_gained += 3;
        }

        if piece.piece_type == PieceType::Pawn && target.is_none() && from.x != to.x {
            let capture_pos = Pos::new(to.x, from.y);
            if let Some(captured) = self.board.get_piece(capture_pos) {
                points_gained += captured.value();
                self.board.set_piece(capture_pos, None);
            }
        }

        self.board.en_passant_target = None;
        if piece.piece_type == PieceType::Pawn && (from.y - to.y).abs() == 2 {
            self.board.en_passant_target = Some(Pos::new(from.x, (from.y + to.y) / 2));
        }

        let promotion_rank = if self.turn == PlayerColor::White {
            7
        } else {
            0
        };
        let is_promoting = (piece.piece_type == PieceType::Pawn
            || piece.piece_type == PieceType::Hawk)
            && to.y == promotion_rank;

        if is_promoting {
            if piece.piece_type == PieceType::Pawn {
                points_gained += 2;
            }
        } else {
            let opponent = self.turn.opposite();
            if self.board.is_in_check(opponent) {
                points_gained += 2;
            }
        }

        match self.turn {
            PlayerColor::White => self.white_points += points_gained,
            PlayerColor::Black => self.black_points += points_gained,
        }

        if is_promoting {
            self.phase = TurnPhase::Promoting(to, piece.piece_type == PieceType::Hawk);
        } else {
            self.end_turn_process();
        }

        self.selected_pos = None;
        self.legal_moves.clear();
    }

    pub fn resolve_promotion(&mut self, new_type: PieceType) {
        if let TurnPhase::Promoting(pos, is_hawk) = self.phase {
            if let Some(mut piece) = self.board.get_piece(pos) {
                piece.piece_type = new_type;
                self.board.set_piece(pos, Some(piece));

                let opponent = self.turn.opposite();
                if self.board.is_in_check(opponent) {
                    match self.turn {
                        PlayerColor::White => self.white_points += 2,
                        PlayerColor::Black => self.black_points += 2,
                    }
                }
            }
            self.end_turn_process();
        }
    }

    fn end_turn_process(&mut self) {
        self.turn = self.turn.opposite();
        self.phase = TurnPhase::Normal;
        self.start_turn();

        if self.board.is_in_check(self.turn) {
            let mut can_move = false;
            for x in 0..8 {
                for y in 0..8 {
                    let pos = Pos::new(x, y);
                    if let Some(p) = self.board.get_piece(pos) {
                        if p.color == self.turn {
                            if !self.board.get_legal_moves(pos).is_empty() {
                                can_move = true;
                                break;
                            }
                        }
                    }
                }
                if can_move {
                    break;
                }
            }
            if !can_move {
                self.winner = Some(self.turn.opposite());
                self.phase = TurnPhase::GameOver;
            }
        }
    }

    pub fn attempt_upgrade(&mut self, pos: Pos) {
        if self.phase != TurnPhase::Normal {
            return;
        }

        if self.board.is_in_check(self.turn) {
            return;
        }

        if let Some(mut piece) = self.board.get_piece(pos) {
            if piece.color == self.turn {
                if let Some(cost) = piece.upgrade_cost() {
                    let points = if self.turn == PlayerColor::White {
                        self.white_points
                    } else {
                        self.black_points
                    };
                    if points >= cost {
                        if let Some(new_type) = piece.upgraded_type() {
                            if self.turn == PlayerColor::White {
                                self.white_points -= cost;
                            } else {
                                self.black_points -= cost;
                            }

                            piece.piece_type = new_type;
                            self.board.set_piece(pos, Some(piece));

                            self.phase = TurnPhase::PostUpgrade(pos);
                            self.selected_pos = None;
                            self.legal_moves.clear();
                        }
                    }
                }
            }
        }
    }
}
