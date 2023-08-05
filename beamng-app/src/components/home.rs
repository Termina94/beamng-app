use crate::hooks::use_websocket::use_websocket;
use beamng_types::{SocketAction, SocketMessage};
use leptos::*;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    let socket = use_websocket(cx);

    let (levels, set_levels) = create_signal::<Vec<String>>(cx, vec![]);
    let (selected_level, set_selected_level) = create_signal(cx, None);

    let set_level = move |level: &String| {
        socket().send(SocketMessage {
            action: SocketAction::SetSelectedLevel(Some(level.to_string())),
        });
    };

    create_effect(cx, move |_| {
        for message in socket().messages {
            match message.action {
                SocketAction::LevelListInit(levels) => {
                    set_levels(levels);
                }
                SocketAction::SetSelectedLevel(level) => {
                    set_selected_level(level);
                }
            }
        }

        if !socket().messages.is_empty() {
            socket.update(|s| s.clear());
        }
    });

    view! { cx,
        <div class="wrapper">
            <div class="level-select">
                <div class="list-title">"BeamNg.drive Level Select"</div>
                <ul class="list-container">
                    {move || levels().into_iter()
                        .map(|n| {
                            let onclick_value = n.clone();
                            view! { cx, <li on:click=move |_| set_level(&onclick_value)  class="list-level-item">{n}</li>}

                        })
                        .collect_view(cx)}
                </ul>
                {move || render_selected_level(cx, selected_level)}
            </div>
        </div>
    }
}

pub fn render_selected_level(
    cx: Scope,
    selected_level: ReadSignal<Option<String>>,
) -> Option<impl IntoView> {
    if selected_level().is_none() {
        return None;
    }

    Some(view! {cx, <div class="selected-level">{move || selected_level().unwrap()}</div>})
}
