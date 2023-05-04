use cozy_chess::{Color, Move};
use proc_macros::Message;
use uuid::Uuid;

pub mod message;

#[derive(Debug, Clone, PartialEq, Message)]
pub enum GameState {
    WaitingForOpponent,
    Ingame { fen: String, color: Color },
}

#[derive(Debug, Clone, PartialEq, Message)]
pub enum ServerMsg {
    ConnectRequired,
    Connected(GameState),
    InvalidSession,
    InvalidLobby,
    OpponentJoined,
    PlayedMove(Move),
    InvalidMove,
}

#[derive(Debug, Clone, PartialEq, Message)]
pub enum ClientMsg {
    Connect {
        lobby_id: Uuid,
        session: Uuid,
    },
    /// Gets sent by the host after they received [`ServerMsg::OpponentJoined`]
    PlayRequest {
        lobby_id: Uuid,
        session: Uuid,
    },
    PlayMove(Move),
}
