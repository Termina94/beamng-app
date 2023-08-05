use std::{borrow::BorrowMut, sync::Arc};

use beamng_types::{SocketMessage, SocketStatus};
use futures_util::{lock::Mutex, stream::SplitSink, SinkExt, StreamExt};
use leptos::*;
use ws_stream_wasm::{WsErr, WsMessage, WsMeta, WsStream};

#[derive(Clone, Default)]
pub struct Websocket {
    pub status: SocketStatus,
    pub messages: Vec<SocketMessage>,
    pub send_action: Option<Action<SocketMessage, Result<(), WsErr>>>,
}

impl Websocket {
    pub fn send(&self, message: SocketMessage) {
        if let Some(action) = self.send_action {
            action.dispatch(message);
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

pub fn use_websocket(cx: Scope) -> RwSignal<Websocket> {
    let socket = create_rw_signal::<Websocket>(cx, Websocket::default());
    let start_websocket = create_action(cx, move |_| handle_websocket(cx, socket));

    create_effect(cx, move |_| match socket().status {
        SocketStatus::Failed => {
            start_websocket.dispatch(cx);
        }
        _ => {}
    });

    start_websocket.dispatch(cx);

    socket
}

async fn handle_websocket(cx: Scope, set_socket: RwSignal<Websocket>) {
    set_socket.update(|ws| ws.status = SocketStatus::Connecting);

    let addr = "ws:192.168.0.40/ws";

    if let Ok((_, wsio)) = WsMeta::connect(addr, None).await {
        let (write, mut read) = wsio.split();
        let writer = Arc::new(Mutex::new(write));

        let send_action = create_action(cx, move |message: &SocketMessage| {
            socket_send(message.clone(), writer.clone())
        });

        set_socket.update(|ws| {
            ws.status = SocketStatus::Connected;
            ws.send_action = Some(send_action);
        });

        while let Some(WsMessage::Text(text)) = read.next().await {
            if let Ok(message) = serde_json::from_str::<SocketMessage>(&text) {
                set_socket.update(|socket| socket.messages.push(message));
            }
        }
    }

    set_timeout(
        move || {
            set_socket.update(|ws| {
                ws.status = SocketStatus::Failed;
                ws.send_action = None;
            });
        },
        std::time::Duration::from_millis(10000),
    );
}

async fn socket_send(
    message: SocketMessage,
    writer: Arc<Mutex<SplitSink<WsStream, WsMessage>>>,
) -> Result<(), WsErr> {
    writer
        .lock()
        .borrow_mut()
        .await
        .send(WsMessage::Text(serde_json::to_string(&message).unwrap()))
        .await
}
