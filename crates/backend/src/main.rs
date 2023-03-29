use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing, Json, Router,
};
use chess::Game;
use common::{HostResponse, JoinResponse, Session};
use log::warn;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

#[derive(Default)]
struct AppState {
    lobbies: HashMap<Uuid, Lobby>,
}

struct Sessions {
    white: Session,
    black: Session,
}

enum Lobby {
    Waiting(Session),
    Playing { game: Game, sessions: Sessions },
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState::default()));

    let app = Router::new()
        .route("/api/host", routing::post(host_game))
        .route("/api/join/:id", routing::post(join_game))
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn host_game(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<HostResponse>, StatusCode> {
    let mut state = match state.lock() {
        Ok(val) => val,
        Err(err) => {
            warn!("state lock is poisoned: {err}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let lobby_code = Uuid::new_v4();
    let session = Uuid::new_v4();
    state.lobbies.insert(lobby_code, Lobby::Waiting(session));

    Ok(Json(HostResponse {
        lobby_id: lobby_code,
        session,
    }))
}

async fn join_game(
    Path(id): Path<Uuid>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Json<JoinResponse>, StatusCode> {
    match state.lock().unwrap().lobbies.get_mut(&id) {
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
