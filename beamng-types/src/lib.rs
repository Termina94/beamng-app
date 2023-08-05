use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SocketMessage {
    pub action: SocketAction,
}

#[derive(Default)]
pub struct AppState {
    pub levels: Vec<String>,
    pub selected_level: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SocketAction {
    SetSelectedLevel(Option<String>),
    LevelListInit(Vec<String>),
}

#[derive(Clone, Default)]
pub enum SocketStatus {
    #[default]
    Init,
    Connecting,
    Connected,
    Failed,
}
