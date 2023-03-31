use axum::extract::ws::Message;
use thiserror::Error;

use super::{ClientMsg, ServerMsg};

impl From<ClientMsg> for Message {
    fn from(val: ClientMsg) -> Self {
        Message::Binary(serde_json::to_vec(&val).unwrap())
    }
}

#[derive(Debug, Error)]
pub enum TryFromError {
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("invalid message type of message '{0:?}'")]
    InvalidMessageType(Message),
}

impl TryFrom<Message> for ClientMsg {
    type Error = TryFromError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => Ok(serde_json::from_str(&text)?),
            Message::Binary(bytes) => Ok(serde_json::from_slice(&bytes)?),
            other => Err(TryFromError::InvalidMessageType(other)),
        }
    }
}

impl From<ServerMsg> for Message {
    fn from(val: ServerMsg) -> Self {
        Message::Binary(serde_json::to_vec(&val).unwrap())
    }
}

impl TryFrom<Message> for ServerMsg {
    type Error = TryFromError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => Ok(serde_json::from_str(&text)?),
            Message::Binary(bytes) => Ok(serde_json::from_slice(&bytes)?),
            other => Err(TryFromError::InvalidMessageType(other)),
        }
    }
}
