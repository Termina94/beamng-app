use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocketMessage {
    pub action: SocketAction,
}

#[derive(Default)]
pub struct AppState {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SocketAction {}

#[derive(Clone, Default)]
pub enum SocketStatus {
    #[default]
    Init,
    Connecting,
    Connected,
    Failed,
}
