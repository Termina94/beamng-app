use crate::hooks::use_websocket::use_websocket;
use beamng_types::{SocketAction, SocketMessage};
use leptos::*;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    // let socket = use_websocket(cx);

    // let (lizard, set_lizard) = create_signal(cx, false);
    // let (gaming, set_gaming) = create_signal(cx, false);

    // let (gaming_loading, set_gaming_loading) = create_signal(cx, false);

    // let toggle_lizard = move |_| {
    //     set_lizard.update(|v| *v = !*v);

    //     socket().send(SocketMessage {
    //         action: SocketAction::SyncToggle(Toggle::Lizard(lizard())),
    //     });
    // };

    // let toggle_lamp = move |_| {
    //     set_gaming_loading(true);
    //     set_gaming.update(|v| *v = !*v);

    //     socket().send(SocketMessage {
    //         action: SocketAction::SyncToggle(Toggle::Gaming(gaming())),
    //     });
    // };

    // create_effect(cx, move |_| {
    //     for message in socket().messages {
    //         match message.action {
    //             SocketAction::SyncToggle(toggle) => match toggle {
    //                 Toggle::Lizard(state) => set_lizard(state),
    //                 Toggle::Gaming(state) => {
    //                     set_gaming(state);
    //                     set_gaming_loading(false);
    //                 }
    //             },
    //             _ => {}
    //         }
    //     }

    //     if !socket().messages.is_empty() {
    //         socket.update(|s| s.clear());
    //     }
    // });

    view! { cx,
        <div class="wrapper">
            "Beamng Level Select"
            // <input prop:checked=lizard on:click=toggle_lizard type="checkbox" id="alias-1" class="hide-checkbox"/>
        </div>
    }
}
