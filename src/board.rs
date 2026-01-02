use crate::constants::*;
use crate::piece::*;
use crate::types::*;

#[derive(Clone, PartialEq)]
pub struct Board {
    pub squares: [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE],
    pub en_passant_target: Option<Pos>,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Self {
            squares: [[None; BOARD_SIZE]; BOARD_SIZE],
            en_passant_target: None,
        };
        board.setup_initial_position();
        board
    }

    fn setup_initial_position(&mut self) {
        self.squares[0][0] = Some(Piece::new(PieceType::Rook, PlayerColor::White));
        self.squares[1][0] = Some(Piece::new(PieceType::Knight, PlayerColor::White));
        self.squares[2][0] = Some(Piece::new(PieceType::Bishop, PlayerColor::White));
        self.squares[3][0] = Some(Piece::new(PieceType::Queen, PlayerColor::White));
        self.squares[4][0] = Some(Piece::new(PieceType::King, PlayerColor::White));
        self.squares[5][0] = Some(Piece::new(PieceType::Bishop, PlayerColor::White));
        self.squares[6][0] = Some(Piece::new(PieceType::Knight, PlayerColor::White));
        self.squares[7][0] = Some(Piece::new(PieceType::Rook, PlayerColor::White));
        for x in 0..8 {
            self.squares[x][1] = Some(Piece::new(PieceType::Pawn, PlayerColor::White));
        }

        self.squares[0][7] = Some(Piece::new(PieceType::Rook, PlayerColor::Black));
        self.squares[1][7] = Some(Piece::new(PieceType::Knight, PlayerColor::Black));
        self.squares[2][7] = Some(Piece::new(PieceType::Bishop, PlayerColor::Black));
        self.squares[3][7] = Some(Piece::new(PieceType::Queen, PlayerColor::Black));
        self.squares[4][7] = Some(Piece::new(PieceType::King, PlayerColor::Black));
        self.squares[5][7] = Some(Piece::new(PieceType::Bishop, PlayerColor::Black));
        self.squares[6][7] = Some(Piece::new(PieceType::Knight, PlayerColor::Black));
        self.squares[7][7] = Some(Piece::new(PieceType::Rook, PlayerColor::Black));
        for x in 0..8 {
            self.squares[x][6] = Some(Piece::new(PieceType::Pawn, PlayerColor::Black));
        }
    }

    pub fn get_piece(&self, pos: Pos) -> Option<Piece> {
        if pos.is_valid() {
            self.squares[pos.x as usize][pos.y as usize]
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, pos: Pos, piece: Option<Piece>) {
        if pos.is_valid() {
            self.squares[pos.x as usize][pos.y as usize] = piece;
        }
    }

    pub fn is_empty(&self, pos: Pos) -> bool {
        self.get_piece(pos).is_none()
    }

    pub fn is_path_clear(&self, from: Pos, to: Pos) -> bool {
        let dx = (to.x - from.x).signum();
        let dy = (to.y - from.y).signum();
        let mut curr = Pos::new(from.x + dx, from.y + dy);

        while curr != to {
            if !self.is_empty(curr) {
                return false;
            }
            curr.x += dx;
            curr.y += dy;
        }
        true
    }

    pub fn get_legal_moves(&self, pos: Pos) -> Vec<Pos> {
        let mut moves = Vec::new();
        let piece = match self.get_piece(pos) {
            Some(p) => p,
            None => return moves,
        };

        let candidates = self.get_pseudo_legal_moves(pos, piece, false);

        for target in candidates {
            let mut temp_board = self.clone();
            temp_board.set_piece(target, Some(piece));
            temp_board.set_piece(pos, None);

            if !temp_board.is_in_check(piece.color) {
                moves.push(target);
            }
        }
        moves
    }

    fn get_pseudo_legal_moves(&self, pos: Pos, piece: Piece, attack_mode: bool) -> Vec<Pos> {
        let mut moves = Vec::new();
        let (x, y) = (pos.x, pos.y);
        let forward_dir = if piece.color == PlayerColor::White {
            1
        } else {
            -1
        };

        match piece.piece_type {
            PieceType::Pawn => {
                if attack_mode {
                    for dx in [-1, 1] {
                        let cap = Pos::new(x + dx, y + forward_dir);
                        if cap.is_valid() {
                            moves.push(cap);
                        }
                    }
                } else {
                    let f1 = Pos::new(x, y + forward_dir);
                    if f1.is_valid() && self.is_empty(f1) {
                        moves.push(f1);
                        if (piece.color == PlayerColor::White && y == 1)
                            || (piece.color == PlayerColor::Black && y == 6)
                        {
                            let f2 = Pos::new(x, y + forward_dir * 2);
                            if self.is_empty(f2) {
                                moves.push(f2);
                            }
                        }
                    }
                    for dx in [-1, 1] {
                        let cap = Pos::new(x + dx, y + forward_dir);
                        if cap.is_valid() {
                            if let Some(target) = self.get_piece(cap) {
                                if target.color != piece.color {
                                    moves.push(cap);
                                }
                            } else if let Some(ep) = self.en_passant_target {
                                if cap == ep {
                                    moves.push(cap);
                                }
                            }
                        }
                    }
                }
            }
            PieceType::Knight => {
                let offsets = [
                    (1, 2),
                    (2, 1),
                    (2, -1),
                    (1, -2),
                    (-1, -2),
                    (-2, -1),
                    (-2, 1),
                    (-1, 2),
                ];
                for (dx, dy) in offsets {
                    let target = Pos::new(x + dx, y + dy);
                    if target.is_valid() {
                        if let Some(p) = self.get_piece(target) {
                            if attack_mode || p.color != piece.color {
                                moves.push(target);
                            }
                        } else {
                            moves.push(target);
                        }
                    }
                }
            }
            PieceType::King => {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let target = Pos::new(x + dx, y + dy);
                        if target.is_valid() {
                            if let Some(p) = self.get_piece(target) {
                                if attack_mode || p.color != piece.color {
                                    moves.push(target);
                                }
                            } else {
                                moves.push(target);
                            }
                        }
                    }
                }
                if !attack_mode && !piece.has_moved && !self.is_in_check(piece.color) {
                    if self.is_path_clear(pos, Pos::new(7, y)) {
                        if let Some(rook) = self.get_piece(Pos::new(7, y)) {
                            if rook.piece_type == PieceType::Rook && !rook.has_moved {
                                let passing_square = Pos::new(x + 1, y);
                                if !self.is_square_attacked(passing_square, piece.color.opposite())
                                {
                                    moves.push(Pos::new(x + 2, y));
                                }
                            }
                        }
                    }
                    if self.is_path_clear(pos, Pos::new(0, y)) {
                        if let Some(rook) = self.get_piece(Pos::new(0, y)) {
                            if rook.piece_type == PieceType::Rook && !rook.has_moved {
                                let passing_square = Pos::new(x - 1, y);
                                if !self.is_square_attacked(passing_square, piece.color.opposite())
                                {
                                    moves.push(Pos::new(x - 2, y));
                                }
                            }
                        }
                    }
                }
            }
            PieceType::Rook => self.add_sliding_moves(
                &mut moves,
                pos,
                piece.color,
                &[(0, 1), (0, -1), (1, 0), (-1, 0)],
                attack_mode,
            ),
            PieceType::Bishop => self.add_sliding_moves(
                &mut moves,
                pos,
                piece.color,
                &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                attack_mode,
            ),
            PieceType::Queen => {
                self.add_sliding_moves(
                    &mut moves,
                    pos,
                    piece.color,
                    &[(0, 1), (0, -1), (1, 0), (-1, 0)],
                    attack_mode,
                );
                self.add_sliding_moves(
                    &mut moves,
                    pos,
                    piece.color,
                    &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    attack_mode,
                );
            }
            PieceType::Hawk => {
                let f1 = Pos::new(x, y + forward_dir);
                if f1.is_valid() && self.is_empty(f1) {
                    moves.push(f1);
                }
                let capture_offsets = [
                    (-1, 0),
                    (1, 0),
                    (0, forward_dir),
                    (-1, forward_dir),
                    (1, forward_dir),
                ];
                for (dx, dy) in capture_offsets {
                    let target = Pos::new(x + dx, y + dy);
                    if target.is_valid() {
                        if let Some(p) = self.get_piece(target) {
                            if attack_mode || p.color != piece.color {
                                moves.push(target);
                            }
                        }
                    }
                }
            }
            PieceType::Elephant => {
                let knight_offsets = [
                    (1, 2),
                    (2, 1),
                    (2, -1),
                    (1, -2),
                    (-1, -2),
                    (-2, -1),
                    (-2, 1),
                    (-1, 2),
                ];
                for (dx, dy) in knight_offsets {
                    let target = Pos::new(x + dx, y + dy);
                    if target.is_valid() {
                        if let Some(p) = self.get_piece(target) {
                            if attack_mode || p.color != piece.color {
                                moves.push(target);
                            }
                        } else {
                            moves.push(target);
                        }
                    }
                }
                let diag_offsets_2 = [(2, 2), (2, -2), (-2, 2), (-2, -2)];
                for (dx, dy) in diag_offsets_2 {
                    let target = Pos::new(x + dx, y + dy);
                    let mid = Pos::new(x + dx / 2, y + dy / 2);
                    if target.is_valid() {
                        // Check obstruction
                        if self.is_empty(mid) {
                            if let Some(p) = self.get_piece(target) {
                                if attack_mode || p.color != piece.color {
                                    moves.push(target);
                                }
                            } else {
                                moves.push(target);
                            }
                        }
                    }
                }
                let diag_offsets_1 = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
                for (dx, dy) in diag_offsets_1 {
                    let target = Pos::new(x + dx, y + dy);
                    if target.is_valid() {
                        if let Some(p) = self.get_piece(target) {
                            if attack_mode || p.color != piece.color {
                                moves.push(target);
                            }
                        } else {
                            moves.push(target);
                        }
                    }
                }
            }
            PieceType::Archbishop => {
                self.add_sliding_moves(
                    &mut moves,
                    pos,
                    piece.color,
                    &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    attack_mode,
                );
                let rook_dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                for (dx, dy) in rook_dirs {
                    let mut curr = Pos::new(x + dx, y + dy);
                    let mut dist = 0;
                    while curr.is_valid() && dist < 1 {
                        if let Some(p) = self.get_piece(curr) {
                            if attack_mode || p.color != piece.color {
                                moves.push(curr);
                            }
                            break;
                        } else {
                            moves.push(curr);
                        }
                        curr.x += dx;
                        curr.y += dy;
                        dist += 1;
                    }
                }
            }
            PieceType::Monarch => {
                self.add_sliding_moves(
                    &mut moves,
                    pos,
                    piece.color,
                    &[(0, 1), (0, -1), (1, 0), (-1, 0)],
                    attack_mode,
                );
                self.add_sliding_moves(
                    &mut moves,
                    pos,
                    piece.color,
                    &[(1, 1), (1, -1), (-1, 1), (-1, -1)],
                    attack_mode,
                );
                let knight_offsets = [
                    (1, 2),
                    (2, 1),
                    (2, -1),
                    (1, -2),
                    (-1, -2),
                    (-2, -1),
                    (-2, 1),
                    (-1, 2),
                ];
                for (dx, dy) in knight_offsets {
                    let target = Pos::new(x + dx, y + dy);
                    if target.is_valid() {
                        if let Some(p) = self.get_piece(target) {
                            if attack_mode || p.color != piece.color {
                                moves.push(target);
                            }
                        } else {
                            moves.push(target);
                        }
                    }
                }
            }
            PieceType::Cannon => {
                let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                for (dx, dy) in dirs {
                    let mut curr = Pos::new(x + dx, y + dy);
                    while curr.is_valid() {
                        if self.is_empty(curr) {
                            moves.push(curr);
                        } else {
                            if let Some(p) = self.get_piece(curr) {
                                if attack_mode || p.color != piece.color {
                                    moves.push(curr);
                                }
                            }

                            let mut next = Pos::new(curr.x + dx, curr.y + dy);
                            while next.is_valid() {
                                if let Some(p) = self.get_piece(next) {
                                    if attack_mode || p.color != piece.color {
                                        moves.push(next);
                                    }
                                    break;
                                }
                                next.x += dx;
                                next.y += dy;
                            }
                            break;
                        }
                        curr.x += dx;
                        curr.y += dy;
                    }
                }
            }
        }
        moves
    }

    fn add_sliding_moves(
        &self,
        moves: &mut Vec<Pos>,
        start: Pos,
        color: PlayerColor,
        dirs: &[(i32, i32)],
        attack_mode: bool,
    ) {
        for (dx, dy) in dirs {
            let mut curr = Pos::new(start.x + dx, start.y + dy);
            while curr.is_valid() {
                if let Some(p) = self.get_piece(curr) {
                    if attack_mode || p.color != color {
                        moves.push(curr);
                    }
                    break;
                } else {
                    moves.push(curr);
                }
                curr.x += dx;
                curr.y += dy;
            }
        }
    }

    pub fn is_in_check(&self, color: PlayerColor) -> bool {
        if let Some(king_pos) = self.find_king(color) {
            self.is_square_attacked(king_pos, color.opposite())
        } else {
            false
        }
    }

    pub fn find_king(&self, color: PlayerColor) -> Option<Pos> {
        for x in 0..8 {
            for y in 0..8 {
                let pos = Pos::new(x, y);
                if let Some(p) = self.get_piece(pos) {
                    if p.piece_type == PieceType::King && p.color == color {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    pub fn is_square_attacked(&self, target: Pos, by_color: PlayerColor) -> bool {
        for x in 0..8 {
            for y in 0..8 {
                let pos = Pos::new(x, y);
                if let Some(p) = self.get_piece(pos) {
                    if p.color == by_color {
                        if self.get_pseudo_legal_moves(pos, p, true).contains(&target) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn has_insufficient_material(&self, white_points: i32, black_points: i32) -> bool {
        let mut white_pieces = Vec::new();
        let mut black_pieces = Vec::new();

        for x in 0..8 {
            for y in 0..8 {
                if let Some(p) = self.get_piece(Pos::new(x, y)) {
                    if p.color == PlayerColor::White {
                        white_pieces.push(p);
                    } else {
                        black_pieces.push(p);
                    }
                }
            }
        }

        !self.has_winning_potential(&white_pieces, white_points)
            && !self.has_winning_potential(&black_pieces, black_points)
    }

    fn has_winning_potential(&self, pieces: &[Piece], points: i32) -> bool {
        let mut bishops = 0;
        let mut knights = 0;

        for p in pieces {
            match p.piece_type {
                PieceType::Pawn
                | PieceType::Rook
                | PieceType::Queen
                | PieceType::Hawk
                | PieceType::Elephant
                | PieceType::Archbishop
                | PieceType::Cannon
                | PieceType::Monarch => return true,
                PieceType::Bishop => bishops += 1,
                PieceType::Knight => knights += 1,
                PieceType::King => {}
            }
        }

        if bishops == 0 && knights == 0 {
            return false;
        }

        if bishops == 1 && knights == 0 {
            return points >= 7;
        }

        if bishops == 0 && knights == 1 {
            return points >= 7;
        }

        if bishops == 0 && knights == 2 {
            return points >= 7;
        }

        true
    }
}
