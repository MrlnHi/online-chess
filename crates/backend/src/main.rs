use axum::{
    extract::{ws::WebSocket, Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use chess::{Board, Game};
use common::{
    http::{HostResponse, JoinResponse},
    ws::{ClientMsg, ServerMsg},
};
use futures::lock::Mutex;
use log::debug;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;

use uuid::Uuid;

struct AppState {
    lobbies: HashMap<Uuid, Lobby>,
    tx: broadcast::Sender<String>,
}

struct Sessions {
    white: Uuid,
    black: Uuid,
}

enum Lobby {
    Waiting(Uuid),
    Playing { game: Game, sessions: Sessions },
}

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100);
    let state = Arc::new(Mutex::new(AppState {
        lobbies: HashMap::new(),
        tx,
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
    // By splitting, we can send and receive at the same time.
    // let (mut sender, mut receiver) = socket.split();

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

    match state.lock().await.lobbies.get(&lobby_id) {
        Some(Lobby::Waiting(sess)) => {
            if *sess == session {
                let _ = socket
                    .send(ServerMsg::Board(Board::default().to_string()).into())
                    .await;
            } else {
                let _ = socket.send(ServerMsg::InvalidSession.into()).await;
            }
        }
        Some(Lobby::Playing { .. }) => {}
        None => {
            let _ = socket.send(ServerMsg::InvalidLobby.into()).await;
            return;
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.lock().await.tx.subscribe();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(_msg) = rx.recv().await {
            // In any websocket error, break loop.
            // if sender.send(Message::Text(msg)).await.is_err() {
            //     break;
            // }
        }
    });

    let _tx = state.lock().await.tx.clone();

    let mut recv_task = tokio::spawn(async move {
        // while let Some(Ok(Message::Text(text))) = receiver.next().await {
        // Add username before message.
        // let _ = tx.send(format!("{}: {}", name, text));
        // }
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
    state
        .lock()
        .await
        .lobbies
        .insert(lobby_code, Lobby::Waiting(session));

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
        Some(lobby) => match *lobby {
            Lobby::Waiting(white) => {
                let black = Uuid::new_v4();
                *lobby = Lobby::Playing {
                    game: Game::new(),
                    sessions: Sessions { white, black },
                };
                Ok(Json(JoinResponse {
                    lobby_id: id,
                    session: black,
                }))
            }
            _ => Err(StatusCode::CONFLICT),
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}
