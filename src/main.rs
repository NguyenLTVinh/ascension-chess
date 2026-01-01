mod assets;
mod board;
mod constants;
mod game;
mod piece;
mod renderer;
mod types;

use assets::*;
use constants::*;
use game::*;
use macroquad::prelude::*;
use types::*;

#[macroquad::main("Ascension Chess")]
async fn main() {
    let mut game = Game::new();
    let assets = Assets::load().await;

    request_new_screen_size(1280.0, 720.0);

    loop {
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let bx = ((mx - BOARD_OFFSET_X) / SQUARE_SIZE).floor() as i32;
            let by = 7 - ((my - BOARD_OFFSET_Y) / SQUARE_SIZE).floor() as i32;

            let pos = Pos::new(bx, by);
            if pos.is_valid() {
                game.select_square(pos);
            }
        }

        if is_key_pressed(KeyCode::U) {
            if let Some(pos) = game.selected_pos {
                game.attempt_upgrade(pos);
            }
        }

        if let TurnPhase::Promoting(_, is_hawk) = game.phase {
            if is_key_pressed(KeyCode::Q) {
                game.resolve_promotion(PieceType::Queen);
            } else if is_key_pressed(KeyCode::R) {
                game.resolve_promotion(PieceType::Rook);
            } else if is_key_pressed(KeyCode::B) {
                game.resolve_promotion(PieceType::Bishop);
            } else if is_key_pressed(KeyCode::N) {
                game.resolve_promotion(PieceType::Knight);
            }

            if is_hawk {
                if is_key_pressed(KeyCode::H) {
                    game.resolve_promotion(PieceType::Hawk);
                } else if is_key_pressed(KeyCode::E) {
                    game.resolve_promotion(PieceType::Elephant);
                } else if is_key_pressed(KeyCode::A) {
                    game.resolve_promotion(PieceType::Archbishop);
                } else if is_key_pressed(KeyCode::C) {
                    game.resolve_promotion(PieceType::Cannon);
                } else if is_key_pressed(KeyCode::K) {
                    game.resolve_promotion(PieceType::Chancellor);
                }
            }
        }

        renderer::draw_game(&game, &assets);

        next_frame().await
    }
}
