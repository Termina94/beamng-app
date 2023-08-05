extern crate poem;
extern crate serde;
extern crate serde_json;

use anyhow::Result;
use async_zip::tokio::read::seek::ZipFileReader;
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
use regex::{Captures, Regex};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
};
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

    let levels = get_levels().await.unwrap();
    let websocket = Arc::new(Mutex::new(WebSocket::default()));
    let state = Arc::new(Mutex::new(AppState {
        levels,
        ..Default::default()
    }));

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

        socket
            .lock()
            .await
            .broadcast(&SocketMessage {
                action: beamng_types::SocketAction::LevelListInit(
                    store.lock().await.levels.clone(),
                ),
            })
            .await;

        socket
            .lock()
            .await
            .broadcast(&SocketMessage {
                action: beamng_types::SocketAction::SetSelectedLevel(
                    store.lock().await.selected_level.clone(),
                ),
            })
            .await;

        while let Some(data) = read.next().await {
            match data {
                Ok(Message::Text(text)) => {
                    if let Ok(message) = serde_json::from_str::<SocketMessage>(&text) {
                        match message.action {
                            beamng_types::SocketAction::SetSelectedLevel(level) => {
                                if let Some(level) = &level {
                                    let _ = update_map_value(level).await;
                                }

                                store.lock().await.selected_level = level.clone();
                                socket
                                    .lock()
                                    .await
                                    .broadcast(&SocketMessage {
                                        action: beamng_types::SocketAction::SetSelectedLevel(level),
                                    })
                                    .await;
                            }
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

async fn update_map_value(new_value: &str) -> io::Result<()> {
    let server_folder = std::env::var("SERVER_FOLDER").expect("SERVER_FOLDER not set");
    let config_file = format!("{}/ServerConfig.toml", server_folder);

    // Read the file content
    let mut file = File::open(&config_file).await?;
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    let mut config = content.parse::<toml::Table>().unwrap();

    let map = config.get_mut("General").unwrap().get_mut("Map").unwrap();

    let re = Regex::new("\"(/levels/)[^/]+(/info.json)\"").unwrap();
    *map = toml::Value::String(
        re.replace(&map.to_string(), |caps: &Captures| {
            format!("{}{}{}", &caps[1], new_value, &caps[2])
        })
        .to_string(),
    );

    // Write the updated content back to the file
    let mut updated_file = File::create(config_file).await?;
    let updated_content = toml::to_string(&config).unwrap();
    updated_file.write_all(updated_content.as_bytes()).await?;

    Ok(())
}

async fn get_levels() -> Result<Vec<String>> {
    let server_folder = std::env::var("SERVER_FOLDER").expect("SERVER_FOLDER not set");
    let level_directory = format!("{}/mods", server_folder);
    let mut mods = tokio::fs::read_dir(level_directory).await?;
    let regex = Regex::new(r"^levels+\/([^/]+)\/$").unwrap();

    let mut levels: Vec<String> = vec![
        "gridmap_v2".to_string(),
        "johnson_valley".to_string(),
        "automation_test_track".to_string(),
        "east_coast_usa".to_string(),
        "hirochi_raceway".to_string(),
        "italy".to_string(),
        "jungle_rock_island".to_string(),
        "industrial".to_string(),
        "small_island".to_string(),
        "smallgrid".to_string(),
        "utah".to_string(),
        "west_coast_usa".to_string(),
        "driver_training".to_string(),
        "derby".to_string(),
    ];

    while let Ok(Some(entry)) = mods.next_entry().await {
        let mut file = File::open(entry.path()).await.unwrap();
        let zip = ZipFileReader::with_tokio(&mut file).await.unwrap();

        zip.file()
            .entries()
            .into_iter()
            .for_each(|ent| match ent.entry().dir().unwrap_or(false) {
                true => {
                    let filename = ent
                        .entry()
                        .filename()
                        .as_str()
                        .unwrap_or_default()
                        .to_string();

                    if regex.is_match(&filename) {
                        let folder = regex.captures(&filename).unwrap().get(1).unwrap().as_str();

                        if !levels.iter().any(|level| level == folder) {
                            levels.push(folder.to_string());
                        }
                    }
                }
                false => {}
            });
    }

    return Ok(levels);
}
