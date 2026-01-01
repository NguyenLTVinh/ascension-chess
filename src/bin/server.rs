use ascension_chess::network::GameMessage;
use ascension_chess::types::PlayerColor;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::time;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

type Tx = mpsc::UnboundedSender<GameMessage>;

const MAX_ROOMS: usize = 1000;
const ROOM_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
const MAX_REQUESTS_PER_WINDOW: u32 = 30;
const MAX_MSG_SIZE: usize = 8 * 1024; // 8KB

struct Room {
    white: Option<Tx>,
    black: Option<Tx>,
    last_active: Instant,
}

struct RateLimiter {
    requests: HashMap<std::net::IpAddr, (u32, Instant)>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            requests: HashMap::new(),
        }
    }

    fn check(&mut self, ip: std::net::IpAddr) -> bool {
        let now = Instant::now();
        let (count, start) = self.requests.entry(ip).or_insert((0, now));

        if now.duration_since(*start) > RATE_LIMIT_WINDOW {
            *count = 1;
            *start = now;
            true
        } else {
            *count += 1;
            *count <= MAX_REQUESTS_PER_WINDOW
        }
    }

    fn cleanup(&mut self) {
        let now = Instant::now();
        self.requests
            .retain(|_, (_, start)| now.duration_since(*start) <= RATE_LIMIT_WINDOW);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on 0.0.0.0:8080");

    let rooms: Arc<Mutex<HashMap<String, Room>>> = Arc::new(Mutex::new(HashMap::new()));
    let rate_limiter = Arc::new(Mutex::new(RateLimiter::new()));

    let rooms_cleanup = rooms.clone();
    let rate_limiter_cleanup = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;

            let mut rooms_guard = rooms_cleanup.lock().await;
            let now = Instant::now();
            rooms_guard.retain(|name, room| {
                let active = now.duration_since(room.last_active) < ROOM_TIMEOUT;
                if !active {
                    println!("Cleaning up abandoned room: {}", name);
                }
                active
            });

            let mut rl_guard = rate_limiter_cleanup.lock().await;
            rl_guard.cleanup();
        }
    });

    loop {
        let (socket, addr) = listener.accept().await?;
        let rooms = rooms.clone();
        let rate_limiter = rate_limiter.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr, rooms, rate_limiter).await {
                eprintln!("Connection error from {}: {}", addr, e);
            }
        });
    }
}

async fn handle_connection(
    socket: TcpStream,
    addr: SocketAddr,
    rooms: Arc<Mutex<HashMap<String, Room>>>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut rl = rate_limiter.lock().await;
        if !rl.check(addr.ip()) {
            return Err("Rate limit exceeded".into());
        }
    }

    let codec = LengthDelimitedCodec::builder()
        .max_frame_length(MAX_MSG_SIZE)
        .new_codec();
    let mut framed = Framed::new(socket, codec);

    let message_bytes = match framed.next().await {
        Some(Ok(bytes)) => bytes,
        None => return Ok(()),
        Some(Err(e)) => return Err(e.into()),
    };

    let message: GameMessage = serde_json::from_slice(&message_bytes)?;

    let (room_name, is_random_creation) = match message {
        GameMessage::Join { room } => {
            if let Some(r) = room {
                if r.len() > 20 || !r.chars().all(char::is_alphanumeric) {
                    return Err("Invalid room name".into());
                }
                (r, false)
            } else {
                let code = format!(
                    "{:06x}",
                    SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                        & 0xFFFFFF
                );
                (code, true)
            }
        }
        _ => return Err("Expected Join message".into()),
    };

    println!(
        "User {} room: {}",
        if is_random_creation {
            "creating"
        } else {
            "joining/creating"
        },
        room_name
    );

    let (tx, mut rx) = mpsc::unbounded_channel();
    let color: PlayerColor;

    {
        let mut rooms_guard = rooms.lock().await;

        let should_create = if is_random_creation {
            if rooms_guard.contains_key(&room_name) {
                let error = GameMessage::Error {
                    message: "Room creation collision. Try again.".into(),
                };
                let bytes = serde_json::to_vec(&error)?;
                framed.send(bytes.into()).await?;
                return Ok(());
            }
            true
        } else {
            !rooms_guard.contains_key(&room_name)
        };

        if should_create {
            if rooms_guard.len() >= MAX_ROOMS {
                let error = GameMessage::Error {
                    message: "Server is full".into(),
                };
                let bytes = serde_json::to_vec(&error)?;
                framed.send(bytes.into()).await?;
                return Ok(());
            }

            rooms_guard.insert(
                room_name.clone(),
                Room {
                    white: Some(tx),
                    black: None,
                    last_active: Instant::now(),
                },
            );
            color = PlayerColor::White;

            let code_msg = GameMessage::RoomCode {
                code: room_name.clone(),
            };
            let bytes = serde_json::to_vec(&code_msg)?;
            framed.send(bytes.into()).await?;
        } else {
            if let Some(room) = rooms_guard.get_mut(&room_name) {
                if room.white.is_none() {
                    room.white = Some(tx);
                    color = PlayerColor::White;
                    room.last_active = Instant::now();
                } else if room.black.is_none() {
                    room.black = Some(tx);
                    color = PlayerColor::Black;
                    room.last_active = Instant::now();
                } else {
                    let error = GameMessage::Error {
                        message: "Room full".into(),
                    };
                    let bytes = serde_json::to_vec(&error)?;
                    framed.send(bytes.into()).await?;
                    return Ok(());
                }
            } else {
                // Should not happen as we checked !rooms_guard.contains_key
                let error = GameMessage::Error {
                    message: "Room not found".into(),
                };
                let bytes = serde_json::to_vec(&error)?;
                framed.send(bytes.into()).await?;
                return Ok(());
            }
        }
    }

    let welcome = GameMessage::Welcome { color };
    let bytes = serde_json::to_vec(&welcome)?;
    framed.send(bytes.into()).await?;

    loop {
        tokio::select! {
            recv_res = rx.recv() => {
                match recv_res {
                    Some(msg) => {
                        let bytes = serde_json::to_vec(&msg)?;
                        framed.send(bytes.into()).await?;
                    }
                    None => break,
                }
            }
            result = framed.next() => {
                match result {
                    Some(Ok(bytes)) => {
                        let msg: GameMessage = serde_json::from_slice(&bytes)?;
                        match msg {
                            GameMessage::Move { .. } |
                            GameMessage::Upgrade { .. } |
                            GameMessage::Promote { .. } => {
                                let mut rooms_guard = rooms.lock().await;
                                if let Some(room) = rooms_guard.get_mut(&room_name) {
                                     room.last_active = Instant::now();
                                     let target = if color == PlayerColor::White {
                                         &room.black
                                     } else {
                                         &room.white
                                     };
                                     if let Some(target_tx) = target {
                                         target_tx.send(msg).ok();
                                     }
                                }
                            },
                            _ => {
                                eprintln!("Ignored invalid message in room {}", room_name);
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
