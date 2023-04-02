use serde::{Deserialize, Serialize};
use uuid::Uuid;

use self::new_types::{ChessMove, Color};

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "reqwasm")]
pub mod reqwasm;

pub mod new_types;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServerMsg {
    PlayRequestRequired,
    PlayResponse { fen: String, color: Color },
    InvalidSession,
    InvalidLobby,
    OpponentJoined,
    PlayedMove(ChessMove),
    InvalidMove,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ClientMsg {
    PlayRequest { lobby_id: Uuid, session: Uuid },
    PlayMove(ChessMove),
}
