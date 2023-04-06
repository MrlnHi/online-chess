use chess::{ChessMove, Color};
use proc_macros::Message;
use uuid::Uuid;

pub mod message;

#[derive(Debug, Clone, PartialEq, Message)]
pub enum ServerMsg {
    PlayRequestRequired,
    PlayResponse { fen: String, color: Color },
    InvalidSession,
    InvalidLobby,
    OpponentJoined,
    PlayedMove(ChessMove),
    InvalidMove,
}

#[derive(Debug, Clone, PartialEq, Message)]
pub enum ClientMsg {
    PlayRequest { lobby_id: Uuid, session: Uuid },
    PlayMove(ChessMove),
}
