extern crate poem;
extern crate serde;
extern crate serde_json;

use beamng_types::{AppState, SocketMessage};
use dotenv::dotenv;
use futures_util::{lock::Mutex, stream::SplitSink, SinkExt, StreamExt};
use poem::{
    endpoint::StaticFilesEndpoint,
    handler,
    listener::TcpListener,
    web::{
        websocket::{Message, WebSocketStream},
        Data,
    },
    EndpointExt, IntoResponse, Route, Server,
};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

#[derive(Default)]
struct WebSocket {
    connections: HashMap<Uuid, Arc<Mutex<SplitSink<WebSocketStream, Message>>>>,
}

type SocketRef = Arc<Mutex<WebSocket>>;

impl WebSocket {
    async fn broadcast(&mut self, message: &SocketMessage) {
        for (_, connection) in &self.connections {
            let _ = connection
                .lock()
                .await
                .send(Message::Text(serde_json::to_string(&message).unwrap()))
                .await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let state = Arc::new(Mutex::new(AppState {
        ..Default::default()
    }));
    let websocket = Arc::new(Mutex::new(WebSocket::default()));

    let app = Route::new()
        .at("/ws", handle_websocket)
        .nest(
            "/",
            StaticFilesEndpoint::new("../beamng-app/dist")
                .show_files_listing()
                .fallback_to_index()
                .index_file("index.html"),
        )
        .data(state.clone())
        .data(websocket.clone());

    let addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");

    println!("Listening on {}", addr);

    Server::new(TcpListener::bind(&addr)).run(app).await
}

#[handler]
async fn handle_websocket(
    ws: poem::web::websocket::WebSocket,
    Data(store): Data<&Arc<Mutex<AppState>>>,
    Data(socket): Data<&SocketRef>,
) -> impl IntoResponse {
    let store = store.clone();
    let socket = socket.clone();

    ws.on_upgrade(|mut ws| async move {
        let (mut write, mut read) = ws.split();
        let socket_ref = Arc::new(Mutex::new(write));
        let uid = Uuid::new_v4();

        socket.lock().await.connections.insert(uid, socket_ref);

        while let Some(data) = read.next().await {
            match data {
                Ok(Message::Text(text)) => {
                    if let Ok(message) = serde_json::from_str::<SocketMessage>(&text) {
                        match message.action {
                            _ => {}
                        }
                    }
                }
                Err(_) | Ok(Message::Close(_)) => {}
                _ => {}
            }
        }
    })
}
