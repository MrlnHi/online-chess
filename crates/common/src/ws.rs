use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "reqwasm")]
pub mod reqwasm;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl From<chess::Color> for Color {
    fn from(value: chess::Color) -> Self {
        match value {
            chess::Color::White => Color::White,
            chess::Color::Black => Color::Black,
        }
    }
}

impl From<Color> for chess::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::White => chess::Color::White,
            Color::Black => chess::Color::Black,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ServerMsg {
    PlayRequestRequired,
    PlayResponse { fen: String, color: Color },
    InvalidSession,
    InvalidLobby,
    OpponentJoined,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ClientMsg {
    PlayRequest { lobby_id: Uuid, session: Uuid },
}
