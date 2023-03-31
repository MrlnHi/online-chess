use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "reqwasm")]
pub mod reqwasm;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServerMsg {
    PlayRequestRequired,
    /// FEN-String of board
    Board(String),
    InvalidSession,
    InvalidLobby,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ClientMsg {
    PlayRequest { lobby_id: Uuid, session: Uuid },
}
