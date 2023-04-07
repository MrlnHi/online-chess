use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{self, get_service},
    Json, Router,
};
use common::{
    http::{HostResponse, JoinResponse},
    ws::{message::Message, ClientMsg, ServerMsg},
};
use cozy_chess::{Board, Color, Move};
use futures::{
    channel::mpsc::{channel, Sender},
    lock::Mutex,
    SinkExt, StreamExt,
};
use log::{debug, error, info, warn};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tower_http::services::{ServeDir, ServeFile};

use uuid::Uuid;

enum PlayerAction {
    PlayMove {
        lobby_id: Uuid,
        color: Color,
        chess_move: Move,
    },
}

struct AppState {
    lobbies: Arc<Mutex<HashMap<Uuid, Lobby>>>,
    tx: Sender<PlayerAction>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Sessions {
    white: Uuid,
    black: Uuid,
}

impl Sessions {
    fn find(&self, session: Uuid) -> Option<Color> {
        if self.white == session {
            Some(Color::White)
        } else if self.black == session {
            Some(Color::Black)
        } else {
            None
        }
    }
}

struct Lobby {
    tx: broadcast::Sender<ServerMsg>,
    state: LobbyState,
}

#[derive(Debug, Clone)]
enum LobbyState {
    Waiting { session: Uuid, color: Color },
    Playing { board: Board, sessions: Sessions },
}

#[tokio::main]
async fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    let lobbies: Arc<Mutex<HashMap<Uuid, Lobby>>> = Arc::default();

    let tx = {
        let (tx, mut rx) = channel(100);
        let lobbies = Arc::clone(&lobbies);
        tokio::spawn(async move {
            while let Some(action) = rx.next().await {
                match action {
                    PlayerAction::PlayMove {
                        lobby_id,
                        color,
                        chess_move,
                    } => match lobbies.lock().await.get_mut(&lobby_id) {
                        Some(lobby) => {
                            match &mut lobby.state {
                                LobbyState::Playing { board, .. } => {
                                    if board.side_to_move() == color {
                                        match board.try_play(chess_move) {
                                            Ok(()) => {
                                                info!("client played move, broadcasting");
                                                _ = lobby.tx.send(ServerMsg::PlayedMove(chess_move));},
                                            Err(err) => {
                                                info!("client sent illegal move {chess_move} ({err}), ignoring");
                                            },
                                        }
                                    } else {
                                        info!("client sent move when it wasn't their turn, ignoring");
                                    }
                                }
                                LobbyState::Waiting { .. } => info!("client sent PlayMove for lobby that is not started yet ({lobby_id}), ignoring"),
                            }
                        }
                        None => {
                            info!("client sent PlayMove for non-existing lobby {lobby_id}, ignoring");
                        }
                    },
                }
            }
        });
        tx
    };

    let state = Arc::new(AppState { lobbies, tx });

    let app = Router::new()
        .route("/api/host", routing::post(host_game))
        .route("/api/join/:id", routing::post(join_game))
        .route("/ws", routing::get(websocket_handler))
        .nest_service(
            "/",
            get_service(ServeDir::new("./assets").fallback(ServeFile::new("./assets/index.html"))),
        )
        .with_state(state);

    info!("starting...");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(mut socket: WebSocket, state: Arc<AppState>) {
    let (lobby_id, session) = if let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                let msg = match Message::from_axum_message(msg) {
                    Ok(msg) => msg,
                    Err(err) => {
                        debug!("error reading client msg: {err}");
                        return;
                    }
                };
                match msg {
                    ClientMsg::PlayRequest { lobby_id, session } => (lobby_id, session),
                    _ => {
                        let Ok(msg) = ServerMsg::PlayRequestRequired.to_axum_message() else {
                            warn!("failed to convert message to axum message: {:?}", ServerMsg::PlayRequestRequired);
                            return;
                        };
                        _ = socket.send(msg).await;
                        return;
                    }
                }
            }
            Err(err) => {
                eprintln!("got error: {err}");
                return;
            }
        }
    } else {
        eprintln!("received nothing");
        return;
    };

    let (tx, color) = match state.lobbies.lock().await.get(&lobby_id) {
        Some(Lobby { state, tx }) => {
            let color = match state {
                LobbyState::Waiting {
                    session: sess,
                    color,
                } => {
                    if *sess == session {
                        _ = socket
                            .send(
                                ServerMsg::PlayResponse {
                                    fen: Board::default().to_string(),
                                    color: *color,
                                }
                                .to_axum_message()
                                .unwrap(),
                            )
                            .await;
                        *color
                    } else {
                        _ = socket
                            .send(ServerMsg::InvalidSession.to_axum_message().unwrap())
                            .await;
                        return;
                    }
                }
                LobbyState::Playing { board, sessions } => {
                    if let Some(color) = sessions.find(session) {
                        _ = socket
                            .send(
                                ServerMsg::PlayResponse {
                                    fen: board.to_string(),
                                    color,
                                }
                                .to_axum_message()
                                .unwrap(),
                            )
                            .await;
                        color
                    } else {
                        _ = socket
                            .send(ServerMsg::InvalidSession.to_axum_message().unwrap())
                            .await;
                        return;
                    }
                }
            };
            (tx.clone(), color)
        }
        None => {
            _ = socket
                .send(ServerMsg::InvalidLobby.to_axum_message().unwrap())
                .await;
            return;
        }
    };

    let mut rx = tx.subscribe();

    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = socket.split();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            let msg = match msg.to_axum_message() {
                Ok(msg) => msg,
                Err(err) => {
                    error!("failed to convert message to axum message: {err:?}");
                    break;
                }
            };
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = {
        let mut tx = state.tx.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                let msg = ClientMsg::from_axum_message(msg);
                match msg {
                    Ok(msg) => match msg {
                        ClientMsg::PlayRequest { .. } => {
                            info!("client sent PlayRequest while already in game, ignoring");
                        }
                        ClientMsg::PlayMove(chess_move) => {
                            tx.send(PlayerAction::PlayMove {
                                lobby_id,
                                color,
                                chess_move,
                            })
                            .await
                            .unwrap();
                        }
                    },
                    Err(err) => {
                        info!("error deserializing client message: {err}");
                        // client is clearly drunk, disconnect
                        break;
                    }
                }
            }
        })
    };
    info!("user joined");

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // User left
    info!("user left");
}

async fn host_game(State(state): State<Arc<AppState>>) -> Result<Json<HostResponse>, StatusCode> {
    let lobby_code = Uuid::new_v4();
    let session = Uuid::new_v4();
    state.lobbies.lock().await.insert(
        lobby_code,
        Lobby {
            // TODO: Increase capacity when introducing spectators
            tx: broadcast::channel(2).0,
            state: LobbyState::Waiting {
                session,
                color: if rand::random() {
                    Color::White
                } else {
                    Color::Black
                },
            },
        },
    );

    Ok(Json(HostResponse {
        lobby_id: lobby_code,
        session,
    }))
}

async fn join_game(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<JoinResponse>, StatusCode> {
    match state.lobbies.lock().await.get_mut(&id) {
        Some(lobby) => match lobby.state {
            LobbyState::Waiting { session, color } => {
                let other = Uuid::new_v4();
                let sessions = if color == Color::White {
                    Sessions {
                        white: session,
                        black: other,
                    }
                } else {
                    Sessions {
                        white: other,
                        black: session,
                    }
                };
                _ = lobby.tx.send(ServerMsg::OpponentJoined);
                lobby.state = LobbyState::Playing {
                    board: Board::default(),
                    sessions,
                };
                Ok(Json(JoinResponse {
                    lobby_id: id,
                    session: other,
                }))
            }
            LobbyState::Playing { .. } => Err(StatusCode::CONFLICT),
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}
