use common::ws::{ClientMsg, ServerMsg};
use futures::{SinkExt, StreamExt};
use log::info;
use reqwasm::websocket::futures::WebSocket;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: Uuid,
    pub session: Uuid,
}

#[function_component]
pub fn Ingame(props: &Props) -> Html {
    // TODO: Use correct url
    let mut ws = WebSocket::open("ws://127.0.0.1:3000/ws").unwrap();
    let id = props.id;
    let session = props.session;
    spawn_local(async move {
        ws.send(
            ClientMsg::PlayRequest {
                lobby_id: id,
                session,
            }
            .into(),
        )
        .await
        .unwrap();
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(msg) => {
                    let msg: ServerMsg = msg.try_into().unwrap();
                    info!("{msg:?}");
                }
                Err(err) => {
                    info!("err: {err}");
                }
            }
        }
        info!("WebSocket Closed");
    });
    html! {
        <p>{format!("Ingame I guess ({})", props.id)}</p>
    }
}
