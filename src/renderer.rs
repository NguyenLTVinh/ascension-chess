use crate::assets::*;
use crate::constants::*;
use crate::game::*;
use crate::types::*;
use macroquad::prelude::*;

pub fn draw_game(game: &Game, assets: &Assets, flipped: bool, offset_x: f32, offset_y: f32) {
    clear_background(LIGHTGRAY);

    // Offset here just to make the indicators behave nicely
    draw_texture_ex(
        &assets.board_texture,
        offset_x - 0.5,
        offset_y - 1.1,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(SQUARE_SIZE * 8.0, SQUARE_SIZE * 8.0)),
            ..Default::default()
        },
    );

    if let Some((from, to)) = game.last_move {
        let (fx, fy) = get_screen_coords(from, flipped, offset_x, offset_y);
        draw_rectangle(fx, fy, SQUARE_SIZE, SQUARE_SIZE, LAST_MOVE_COLOR);
        let (tx, ty) = get_screen_coords(to, flipped, offset_x, offset_y);
        draw_rectangle(tx, ty, SQUARE_SIZE, SQUARE_SIZE, LAST_MOVE_COLOR);
    }

    if game.board.is_in_check(game.turn) {
        if let Some(king_pos) = game.board.find_king(game.turn) {
            let (kx, ky) = get_screen_coords(king_pos, flipped, offset_x, offset_y);
            draw_rectangle(kx, ky, SQUARE_SIZE, SQUARE_SIZE, CHECK_COLOR);
        }
    }

    if let Some(pos) = game.selected_pos {
        let (sx, sy) = get_screen_coords(pos, flipped, offset_x, offset_y);
        draw_rectangle(sx, sy, SQUARE_SIZE, SQUARE_SIZE, SELECTION_COLOR);
    }

    for pos in &game.legal_moves {
        let (sx, sy) = get_screen_coords(*pos, flipped, offset_x, offset_y);
        let center_x = sx + SQUARE_SIZE / 2.0;
        let center_y = sy + SQUARE_SIZE / 2.0;

        if let Some(_piece) = game.board.get_piece(*pos) {
            let size = 15.0;
            draw_triangle(
                vec2(sx, sy),
                vec2(sx + size, sy),
                vec2(sx, sy + size),
                CAPTURE_HINT_COLOR,
            );
            draw_triangle(
                vec2(sx + SQUARE_SIZE, sy),
                vec2(sx + SQUARE_SIZE - size, sy),
                vec2(sx + SQUARE_SIZE, sy + size),
                CAPTURE_HINT_COLOR,
            );
            draw_triangle(
                vec2(sx, sy + SQUARE_SIZE),
                vec2(sx + size, sy + SQUARE_SIZE),
                vec2(sx, sy + SQUARE_SIZE - size),
                CAPTURE_HINT_COLOR,
            );
            draw_triangle(
                vec2(sx + SQUARE_SIZE, sy + SQUARE_SIZE),
                vec2(sx + SQUARE_SIZE - size, sy + SQUARE_SIZE),
                vec2(sx + SQUARE_SIZE, sy + SQUARE_SIZE - size),
                CAPTURE_HINT_COLOR,
            );
        } else {
            draw_circle(center_x, center_y, SQUARE_SIZE * 0.15, MOVE_HINT_COLOR);
        }
    }

    for x in 0..8 {
        for y in 0..8 {
            let pos = Pos::new(x, y);
            if let Some(piece) = game.board.get_piece(pos) {
                if let Some(tex) = assets.textures.get(&(piece.piece_type, piece.color)) {
                    let (sx, sy) = get_screen_coords(pos, flipped, offset_x, offset_y);
                    draw_texture_ex(
                        tex,
                        sx,
                        sy,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(SQUARE_SIZE, SQUARE_SIZE)),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    draw_ui(game, offset_x, offset_y);
    draw_rules(assets, offset_x, offset_y);
}

fn get_screen_coords(pos: Pos, flipped: bool, offset_x: f32, offset_y: f32) -> (f32, f32) {
    let effective_x = if flipped { 7 - pos.x } else { pos.x };
    let effective_y = if flipped { pos.y } else { 7 - pos.y };

    let sx = offset_x + effective_x as f32 * SQUARE_SIZE;
    let sy = offset_y + effective_y as f32 * SQUARE_SIZE;
    (sx, sy)
}

fn draw_rules(assets: &Assets, offset_x: f32, offset_y: f32) {
    let start_x = offset_x - 300.0;
    let mut y = offset_y + 20.0;

    draw_text("Ascension Rules", start_x, y, 30.0, BLACK);
    y += 40.0;

    draw_text("Points:", start_x, y, 25.0, BLACK);
    y += 25.0;
    draw_text("Turn:+1, Capture:+Value", start_x, y, 20.0, DARKGRAY);
    y += 20.0;
    draw_text("Check/Promo:+2, Castle:+3", start_x, y, 20.0, DARKGRAY);
    y += 40.0;

    draw_text("Ascended Pieces:", start_x, y, 25.0, BLACK);
    y += 30.0;

    let rules = [
        (
            PieceType::Hawk,
            format!("Hawk Warrior (Val= {} pts)", VAL_HAWK),
            "Move: Fwd 1\nCapture: Fwd/Diag/Side 1",
        ),
        (
            PieceType::Elephant,
            format!("War Elephant (Val= {} pts)", VAL_ELEPHANT),
            "Move: Knight + Diag 1 or 2\nCapture: Knight + Diag 1 or 2",
        ),
        (
            PieceType::Archbishop,
            format!("Archbishop (Val= {} pts)", VAL_ARCHBISHOP),
            "Move: Diag + Fwd/Side 1\nCapture:  Diag + Fwd/Side 1",
        ),
        (
            PieceType::Cannon,
            format!("Cannon (Val= {} pts)", VAL_CANNON),
            "Move: Fwd/Side\nCapture: Fwd/Side + Jump One",
        ),
        (
            PieceType::Monarch,
            format!("Monarch (Val= {} pts)", VAL_MONARCH),
            "Move: Fwd/Diag/Side + Knight\nCapture: Fwd/Diag/Side + Knight",
        ),
    ];

    for (pt, name, desc) in rules.iter() {
        if let Some(tex) = assets.textures.get(&(*pt, PlayerColor::White)) {
            draw_texture_ex(
                tex,
                start_x,
                y - 25.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(36.0, 36.0)),
                    ..Default::default()
                },
            );
        }

        draw_text(name, start_x + 40.0, y, 22.0, BLACK);
        y += 25.0;

        for line in desc.split('\n') {
            draw_text(line, start_x + 40.0, y, 18.0, DARKGRAY);
            y += 20.0;
        }
        y += 15.0;
    }
}

fn draw_ui(game: &Game, offset_x: f32, offset_y: f32) {
    let ui_x = offset_x + SQUARE_SIZE * 8.0 + 20.0;
    let mut ui_y = offset_y;

    let turn_text = format!("Turn: {:?} ({:?})", game.turn, game.phase);
    draw_text(&turn_text, ui_x, ui_y + 20.0, 30.0, BLACK);
    ui_y += 50.0;

    draw_text(
        &format!("White Points: {}", game.white_points),
        ui_x,
        ui_y,
        25.0,
        BLACK,
    );
    ui_y += 30.0;
    draw_text(
        &format!("Black Points: {}", game.black_points),
        ui_x,
        ui_y,
        25.0,
        BLACK,
    );
    ui_y += 50.0;

    draw_text("Ascend (Click Piece + U Key):", ui_x, ui_y, 20.0, BLACK);
    ui_y += 25.0;
    draw_text(
        "P -> Hawk Warrior (H): Costs 5 pts",
        ui_x,
        ui_y,
        20.0,
        DARKGRAY,
    );
    ui_y += 20.0;
    draw_text(
        "N -> War Elephant (E): Costs 7 pts",
        ui_x,
        ui_y,
        20.0,
        DARKGRAY,
    );
    ui_y += 20.0;
    draw_text(
        "B -> Archbishop (A): Costs 7 pts",
        ui_x,
        ui_y,
        20.0,
        DARKGRAY,
    );
    ui_y += 20.0;
    draw_text("R -> Cannon (C): Costs 8 pts", ui_x, ui_y, 20.0, DARKGRAY);
    ui_y += 20.0;
    draw_text("Q -> Monarch (M): Costs 12 pts", ui_x, ui_y, 20.0, DARKGRAY);
    ui_y += 40.0;

    if let TurnPhase::Promoting(_, is_hawk) = game.phase {
        draw_text("PROMOTION! Press Key:", ui_x, ui_y, 30.0, RED);
        ui_y += 30.0;
        if is_hawk {
            draw_text("Promote to ANY piece", ui_x, ui_y, 20.0, RED);
        } else {
            draw_text("Promote to Q, R, B, N", ui_x, ui_y, 20.0, RED);
        }
    }

    if let Some(result) = &game.result {
        match result {
            GameResult::Win(winner) => {
                draw_text(
                    &format!("WINNER: {:?}", winner),
                    ui_x,
                    ui_y + 50.0,
                    40.0,
                    RED,
                );
            }
            GameResult::Draw(reason) => {
                draw_text("DRAW", ui_x, ui_y + 50.0, 40.0, RED);
                let reason_text = match reason {
                    DrawReason::Stalemate => "Stalemate",
                    DrawReason::ThreeFoldRepetition => "3-Fold Repetition",
                    DrawReason::InsufficientMaterial => "Insufficient Material",
                    DrawReason::FiftyMoveRule => "50 Move Rule",
                };
                draw_text(reason_text, ui_x, ui_y + 90.0, 30.0, RED);
            }
        }
    }
}
