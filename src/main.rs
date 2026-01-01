use ascension_chess::assets::*;
use ascension_chess::constants::*;
use ascension_chess::game::*;
use ascension_chess::network::GameMessage;
use ascension_chess::renderer;
use ascension_chess::types::*;
use clap::Parser;
use futures::{SinkExt, StreamExt};
use macroquad::prelude::*;
use std::sync::mpsc;
use std::thread;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    password: Option<String>,

    #[arg(long, default_value = "127.0.0.1:8080")]
    server: String,
}

#[macroquad::main("Ascension Chess")]
async fn main() {
    let args = Args::parse();
    let mut game = Game::new();
    let assets = Assets::load().await;

    request_new_screen_size(1280.0, 720.0);

    let mut is_online = false;
    let mut my_color = PlayerColor::White;
    let mut flipped = false;
    let mut connected = false;

    let (game_tx, game_rx) = mpsc::channel::<GameMessage>();
    let (net_tx, mut net_rx_tokio) = tokio::sync::mpsc::unbounded_channel::<GameMessage>();

    if let Some(password) = args.password {
        is_online = true;
        let server_addr = args.server.clone();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match TcpStream::connect(server_addr).await {
                    Ok(socket) => {
                        let mut framed = Framed::new(socket, LengthDelimitedCodec::new());
                        let join_msg = GameMessage::Join { room: password };
                        let bytes = serde_json::to_vec(&join_msg).unwrap();
                        if let Err(_) = framed.send(bytes.into()).await {
                             game_tx.send(GameMessage::Error { message: "Failed to send join".into() }).ok();
                             return;
                        }

                        loop {
                            tokio::select! {
                                Some(msg) = net_rx_tokio.recv() => {
                                    if let Ok(bytes) = serde_json::to_vec(&msg) {
                                        framed.send(bytes.into()).await.ok();
                                    }
                                }
                                result = framed.next() => {
                                    match result {
                                        Some(Ok(bytes)) => {
                                            if let Ok(msg) = serde_json::from_slice::<GameMessage>(&bytes) {
                                                game_tx.send(msg).ok();
                                            }
                                        }
                                        _ => {
                                            game_tx.send(GameMessage::OpponentDisconnected).ok();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        game_tx.send(GameMessage::Error { message: e.to_string() }).ok();
                    }
                }
            });
        });
    } else {
        connected = true;
    }

    loop {
        while let Ok(msg) = game_rx.try_recv() {
            match msg {
                GameMessage::Welcome { color } => {
                    my_color = color;
                    flipped = color == PlayerColor::Black;
                    connected = true;
                }
                GameMessage::Move { from, to } => {
                    game.make_move(from, to);
                }
                GameMessage::Upgrade { pos } => {
                    game.attempt_upgrade(pos);
                }
                GameMessage::Promote { piece_type } => {
                    game.resolve_promotion(piece_type);
                }
                GameMessage::Error { message } => {
                    println!("Error: {}", message);
                }
                GameMessage::OpponentDisconnected => {
                    println!("Opponent disconnected");
                }
                _ => {}
            }
        }

        if !connected && is_online {
            clear_background(LIGHTGRAY);
            draw_text("Connecting...", 100.0, 100.0, 40.0, BLACK);
            next_frame().await;
            continue;
        }

        let can_play = !is_online || game.turn == my_color;

        if is_mouse_button_pressed(MouseButton::Left) && can_play {
            let (mx, my) = mouse_position();

            let visual_x = ((mx - BOARD_OFFSET_X) / SQUARE_SIZE).floor() as i32;
            let visual_y = ((my - BOARD_OFFSET_Y) / SQUARE_SIZE).floor() as i32;

            let bx = if flipped { 7 - visual_x } else { visual_x };
            let by = if flipped { visual_y } else { 7 - visual_y };

            let pos = Pos::new(bx, by);
            if pos.is_valid() {
                let prev_selected = game.selected_pos;
                let is_move = if let Some(_) = prev_selected {
                    game.legal_moves.contains(&pos)
                } else {
                    false
                };

                game.select_square(pos);

                if is_move {
                    if let Some(from) = prev_selected {
                        if is_online {
                            net_tx.send(GameMessage::Move { from, to: pos }).ok();
                        }
                    }
                }
            }
        }

        if is_key_pressed(KeyCode::U) && can_play {
            if let Some(pos) = game.selected_pos {
                game.attempt_upgrade(pos);
                if let TurnPhase::PostUpgrade(p) = game.phase {
                    if p == pos {
                        if is_online {
                            net_tx.send(GameMessage::Upgrade { pos }).ok();
                        }
                    }
                }
            }
        }

        if let TurnPhase::Promoting(_, is_hawk) = game.phase {
            let mut promoted_type = None;

            if can_play {
                if is_key_pressed(KeyCode::Q) {
                    promoted_type = Some(PieceType::Queen);
                } else if is_key_pressed(KeyCode::R) {
                    promoted_type = Some(PieceType::Rook);
                } else if is_key_pressed(KeyCode::B) {
                    promoted_type = Some(PieceType::Bishop);
                } else if is_key_pressed(KeyCode::N) {
                    promoted_type = Some(PieceType::Knight);
                }

                if is_hawk {
                    if is_key_pressed(KeyCode::H) {
                        promoted_type = Some(PieceType::Hawk);
                    } else if is_key_pressed(KeyCode::E) {
                        promoted_type = Some(PieceType::Elephant);
                    } else if is_key_pressed(KeyCode::A) {
                        promoted_type = Some(PieceType::Archbishop);
                    } else if is_key_pressed(KeyCode::C) {
                        promoted_type = Some(PieceType::Cannon);
                    } else if is_key_pressed(KeyCode::M) {
                        promoted_type = Some(PieceType::Monarch);
                    }
                }
            }

            if let Some(pt) = promoted_type {
                game.resolve_promotion(pt);
                if is_online {
                    net_tx.send(GameMessage::Promote { piece_type: pt }).ok();
                }
            }
        }

        renderer::draw_game(&game, &assets, flipped);

        next_frame().await
    }
}
