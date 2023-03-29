use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Session = Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HostResponse {
    pub lobby_id: Uuid,
    pub session: Session,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct JoinResponse {
    pub lobby_id: Uuid,
    pub session: Session,
}
