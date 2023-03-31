use reqwasm::websocket::Message;

use super::{ClientMsg, ServerMsg};

impl From<ClientMsg> for Message {
    fn from(val: ClientMsg) -> Self {
        Message::Bytes(serde_json::to_vec(&val).unwrap())
    }
}

impl TryFrom<Message> for ClientMsg {
    type Error = serde_json::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => serde_json::from_str(&text),
            Message::Bytes(bytes) => serde_json::from_slice(&bytes),
        }
    }
}

impl From<ServerMsg> for Message {
    fn from(val: ServerMsg) -> Self {
        Message::Bytes(serde_json::to_vec(&val).unwrap())
    }
}

impl TryFrom<Message> for ServerMsg {
    type Error = serde_json::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => serde_json::from_str(&text),
            Message::Bytes(bytes) => serde_json::from_slice(&bytes),
        }
    }
}
