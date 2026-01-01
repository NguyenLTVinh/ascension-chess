use ascension_chess::network::GameMessage;
use ascension_chess::types::PlayerColor;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

type Tx = mpsc::UnboundedSender<GameMessage>;

struct Room {
    white: Option<Tx>,
    black: Option<Tx>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on 0.0.0.0:8080");

    let rooms: Arc<Mutex<HashMap<String, Room>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await?;
        let rooms = rooms.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, rooms).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    socket: TcpStream,
    rooms: Arc<Mutex<HashMap<String, Room>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut framed = Framed::new(socket, LengthDelimitedCodec::new());

    let message_bytes = match framed.next().await {
        Some(Ok(bytes)) => bytes,
        _ => return Err("Connection closed before join".into()),
    };

    let message: GameMessage = serde_json::from_slice(&message_bytes)?;

    let room_name = match message {
        GameMessage::Join { room } => room,
        _ => return Err("Expected Join message".into()),
    };

    println!("User joining room: {}", room_name);

    let (tx, mut rx) = mpsc::unbounded_channel();
    let color: PlayerColor;

    {
        let mut rooms_guard = rooms.lock().await;
        let room = rooms_guard.entry(room_name.clone()).or_insert(Room {
            white: None,
            black: None,
        });

        if room.white.is_none() {
            room.white = Some(tx);
            color = PlayerColor::White;
        } else if room.black.is_none() {
            room.black = Some(tx);
            color = PlayerColor::Black;
        } else {
            let error = GameMessage::Error {
                message: "Room full".into(),
            };
            let bytes = serde_json::to_vec(&error)?;
            framed.send(bytes.into()).await?;
            return Ok(());
        }
    }

    let welcome = GameMessage::Welcome { color };
    let bytes = serde_json::to_vec(&welcome)?;
    framed.send(bytes.into()).await?;

    loop {
        tokio::select! {
            Some(msg) = rx.recv() => {
                let bytes = serde_json::to_vec(&msg)?;
                framed.send(bytes.into()).await?;
            }
            result = framed.next() => {
                match result {
                    Some(Ok(bytes)) => {
                        let msg: GameMessage = serde_json::from_slice(&bytes)?;
                        let rooms_guard = rooms.lock().await;
                        if let Some(room) = rooms_guard.get(&room_name) {
                             let target = if color == PlayerColor::White {
                                 &room.black
                             } else {
                                 &room.white
                             };
                             if let Some(target_tx) = target {
                                 target_tx.send(msg).ok();
                             }
                        }
                    }
                    _ => break,
                }
            }
        }
    }

    let mut rooms_guard = rooms.lock().await;
    if let Some(room) = rooms_guard.get_mut(&room_name) {
        if color == PlayerColor::White {
            room.white = None;
            if let Some(black_tx) = &room.black {
                black_tx.send(GameMessage::OpponentDisconnected).ok();
            }
        } else {
            room.black = None;
            if let Some(white_tx) = &room.white {
                white_tx.send(GameMessage::OpponentDisconnected).ok();
            }
        }
        if room.white.is_none() && room.black.is_none() {
            rooms_guard.remove(&room_name);
        }
    }

    println!("User left room: {}", room_name);
    Ok(())
}
