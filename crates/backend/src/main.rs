use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use chess::{Board, Color, Game};
use common::{
    http::{HostResponse, JoinResponse},
    ws::{ClientMsg, ServerMsg},
};
use futures::{lock::Mutex, SinkExt, StreamExt};
use log::debug;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;

use uuid::Uuid;

struct AppState {
    lobbies: HashMap<Uuid, Lobby>,
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
    players: Players,
}

#[derive(Debug, Clone)]
enum Players {
    Waiting { session: Uuid, color: Color },
    Playing { game: Game, sessions: Sessions },
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState {
        lobbies: HashMap::new(),
    }));

    let app = Router::new()
        .route("/api/host", routing::post(host_game))
        .route("/api/join/:id", routing::post(join_game))
        .route("/ws", routing::get(websocket_handler))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(mut socket: WebSocket, state: Arc<Mutex<AppState>>) {
    let (lobby_id, session) = if let Some(msg) = socket.recv().await {
        match msg {
            Ok(msg) => {
                let msg: ClientMsg = match msg.try_into() {
                    Ok(msg) => msg,
                    Err(err) => {
                        debug!("error reading client msg: {err}");
                        return;
                    }
                };
                match msg {
                    ClientMsg::PlayRequest { lobby_id, session } => (lobby_id, session),
                    _ => {
                        let _ = socket.send(ServerMsg::PlayRequestRequired.into()).await;
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

    let tx = match state.lock().await.lobbies.get(&lobby_id) {
        Some(Lobby { players, tx }) => {
            match players {
                Players::Waiting {
                    session: sess,
                    color,
                } => {
                    if *sess == session {
                        let _ = socket
                            .send(
                                ServerMsg::PlayResponse {
                                    fen: Board::default().to_string(),
                                    color: (*color).into(),
                                }
                                .into(),
                            )
                            .await;
                    } else {
                        let _ = socket.send(ServerMsg::InvalidSession.into()).await;
                    }
                }
                Players::Playing { game, sessions } => {
                    if let Some(color) = sessions.find(session) {
                        let _ = socket
                            .send(
                                ServerMsg::PlayResponse {
                                    fen: game.current_position().to_string(),
                                    color: color.into(),
                                }
                                .into(),
                            )
                            .await;
                    } else {
                        let _ = socket.send(ServerMsg::InvalidSession.into()).await;
                    }
                }
            }
            tx.clone()
        }
        None => {
            let _ = socket.send(ServerMsg::InvalidLobby.into()).await;
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
            if sender.send(msg.into()).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let msg = ClientMsg::try_from(msg);
            match msg {
                Ok(msg) => {
                    debug!("received {msg:?} from client");
                }
                Err(err) => {
                    debug!("error deserializing client message: {err}");
                    // client is clearly drunk, disconnect
                    break;
                }
            }
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    // User left
}

async fn host_game(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<HostResponse>, StatusCode> {
    let lobby_code = Uuid::new_v4();
    let session = Uuid::new_v4();
    state.lock().await.lobbies.insert(
        lobby_code,
        Lobby {
            // TODO: Increase capacity when introducing spectators
            tx: broadcast::channel(2).0,
            players: Players::Waiting {
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
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<JoinResponse>, StatusCode> {
    match state.lock().await.lobbies.get_mut(&id) {
        Some(lobby) => match lobby.players {
            Players::Waiting { session, color } => {
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
                let _ = lobby.tx.send(ServerMsg::OpponentJoined);
                lobby.players = Players::Playing {
                    game: Game::new(),
                    sessions,
                };
                Ok(Json(JoinResponse {
                    lobby_id: id,
                    session: other,
                }))
            }
            _ => Err(StatusCode::CONFLICT),
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}
