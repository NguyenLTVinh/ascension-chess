use crate::assets::*;
use crate::constants::*;
use crate::game::*;
use crate::types::*;
use macroquad::prelude::*;

pub fn draw_game(game: &Game, assets: &Assets, flipped: bool) {
    clear_background(LIGHTGRAY);

    draw_texture_ex(
        &assets.board_texture,
        BOARD_OFFSET_X,
        BOARD_OFFSET_Y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(SQUARE_SIZE * 8.0, SQUARE_SIZE * 8.0)),
            ..Default::default()
        },
    );

    if let Some((from, to)) = game.last_move {
        let (fx, fy) = get_screen_coords(from, flipped);
        draw_rectangle(fx, fy, SQUARE_SIZE, SQUARE_SIZE, LAST_MOVE_COLOR);
        let (tx, ty) = get_screen_coords(to, flipped);
        draw_rectangle(tx, ty, SQUARE_SIZE, SQUARE_SIZE, LAST_MOVE_COLOR);
    }

    if game.board.is_in_check(game.turn) {
        if let Some(king_pos) = game.board.find_king(game.turn) {
            let (kx, ky) = get_screen_coords(king_pos, flipped);
            draw_rectangle(kx, ky, SQUARE_SIZE, SQUARE_SIZE, CHECK_COLOR);
        }
    }

    if let Some(pos) = game.selected_pos {
        let (sx, sy) = get_screen_coords(pos, flipped);
        draw_rectangle(sx, sy, SQUARE_SIZE, SQUARE_SIZE, SELECTION_COLOR);
    }

    for pos in &game.legal_moves {
        let (sx, sy) = get_screen_coords(*pos, flipped);
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
                    let (sx, sy) = get_screen_coords(pos, flipped);
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

    draw_ui(game);
}

fn get_screen_coords(pos: Pos, flipped: bool) -> (f32, f32) {
    let effective_x = if flipped { 7 - pos.x } else { pos.x };
    let effective_y = if flipped { pos.y } else { 7 - pos.y };

    let sx = BOARD_OFFSET_X + effective_x as f32 * SQUARE_SIZE;
    let sy = BOARD_OFFSET_Y + effective_y as f32 * SQUARE_SIZE;
    (sx, sy)
}

fn draw_ui(game: &Game) {
    let ui_x = BOARD_OFFSET_X + SQUARE_SIZE * 8.0 + 20.0;
    let mut ui_y = BOARD_OFFSET_Y;

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

    draw_text("Upgrades (Click Piece + U Key):", ui_x, ui_y, 20.0, BLACK);
    ui_y += 25.0;
    draw_text("P -> Hawk (H): 5 pts", ui_x, ui_y, 20.0, DARKGRAY);
    ui_y += 20.0;
    draw_text("N -> Elephant (E): 7 pts", ui_x, ui_y, 20.0, DARKGRAY);
    ui_y += 20.0;
    draw_text("B -> Archbishop (A): 7 pts", ui_x, ui_y, 20.0, DARKGRAY);
    ui_y += 20.0;
    draw_text("R -> Cannon (C): 8 pts", ui_x, ui_y, 20.0, DARKGRAY);
    ui_y += 20.0;
    draw_text("Q -> Monarch (M): 12 pts", ui_x, ui_y, 20.0, DARKGRAY);
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
