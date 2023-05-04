use common::ws::{message::Message, ClientMsg, ServerMsg};
use futures::{SinkExt, StreamExt};
use log::{info, warn};
use reqwasm::websocket::futures::WebSocket;
use uuid::Uuid;
use yew::{platform::spawn_local, prelude::*};
use yew_router::prelude::*;

use crate::Route;

#[derive(Debug, Clone)]
pub struct WaitingForOpponent;

#[derive(Debug, Clone, Copy, PartialEq, Properties)]
pub struct Props {
    pub id: Uuid,
    pub session: Uuid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    ReceivedMsg(ServerMsg),
}

impl Component for WaitingForOpponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let hostname = web_sys::window().unwrap().location().hostname().unwrap();
        let ws = WebSocket::open(&format!("ws://{hostname}:3000/ws")).unwrap();
        let (mut send, mut receive) = ws.split();

        {
            let id = ctx.props().id;
            let session = ctx.props().session;

            let link = ctx.link().clone();
            spawn_local(async move {
                send.send(
                    ClientMsg::Connect {
                        lobby_id: id,
                        session,
                    }
                    .to_reqwasm_message()
                    .unwrap(),
                )
                .await
                .unwrap();
                info!("sent PlayRequest");
                spawn_local(async move {
                    while let Some(msg) = receive.next().await {
                        match msg {
                            Ok(msg) => match Message::from_reqwasm_message(msg) {
                                Ok(msg) => {
                                    link.send_message(Msg::ReceivedMsg(msg));
                                }
                                Err(err) => {
                                    info!("error deserializing message: {err}");
                                }
                            },
                            Err(err) => {
                                info!("error receiving message: {err}");
                            }
                        }
                    }
                });
            });
        }

        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        fn copy(str: &str) {
            #[cfg(web_sys_unstable_apis)]
            {
                let clipboard = web_sys::window().unwrap().navigator().clipboard().unwrap();
                _ = clipboard.write_text(str);
            }

            #[cfg(not(web_sys_unstable_apis))]
            compile_error!("enable web_sys_unstable_apis");
        }

        let copy_link = {
            let id = ctx.props().id;
            let origin = web_sys::window().unwrap().location().origin().unwrap();
            Callback::from(move |_e| {
                let route = Route::Join { id };
                copy(&format!("{origin}{}", route.to_path()));
            })
        };
        html! {
            <>
                <p> {"Waiting for your opponent..."} </p>
                <button onclick={copy_link}> {"Copy game link"} </button>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReceivedMsg(msg) => match msg {
                ServerMsg::OpponentJoined => {
                    ctx.link().navigator().unwrap().push(&Route::Ingame {
                        id: ctx.props().id,
                        session: ctx.props().session,
                    });
                    false
                }
                _ => {
                    warn!("received {msg:?}, should only receive OpponentJoined");
                    false
                }
            },
        }
    }
}
