use common::ws::message::Message;
use std::str::FromStr;

use crate::Route;

use super::board::Board;

use common::ws::{ClientMsg, ServerMsg};
use cozy_chess::{Board as ChessBoard, Color, Move};
use futures::{
    channel::mpsc::{channel, Sender},
    SinkExt, StreamExt,
};
use log::{info, warn};
use reqwasm::websocket::futures::WebSocket;
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::{html, Callback, Component, Html, Properties};
use yew_router::scope_ext::RouterScopeExt;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: Uuid,
    pub session: Uuid,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub board: ChessBoard,
    pub color: Color,
}

pub struct Ingame {
    game: Option<Game>,
    tx: Sender<ClientMsg>,
}

pub enum Msg {
    ReceivedMsg(ServerMsg),
}

impl Component for Ingame {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        // TODO: Use correct url
        let ws = WebSocket::open("ws://127.0.0.1:3000/ws").unwrap();
        let (mut send, mut receive) = ws.split();

        let (tx, mut rx) = channel::<ClientMsg>(0);

        {
            let id = ctx.props().id;
            let session = ctx.props().session;

            let link = ctx.link().clone();
            spawn_local(async move {
                send.send(
                    ClientMsg::PlayRequest {
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
                spawn_local(async move {
                    while let Some(msg) = rx.next().await {
                        send.send(msg.to_reqwasm_message().unwrap()).await.unwrap();
                    }
                });
            });
        }

        Self { tx, game: None }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> Html {
        if let Some(Game { board, color }) = self.game.clone() {
            let play_move = {
                let tx = self.tx.clone();
                Callback::from(move |chess_move: Move| {
                    let mut tx = tx.clone();
                    spawn_local(async move {
                        tx.send(ClientMsg::PlayMove(chess_move)).await.unwrap();
                    });
                })
            };
            html! {
                <Board {board} {color} {play_move}/>
            }
        } else {
            html! {
                <p>{"Connecting..."}</p>
            }
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ReceivedMsg(msg) => match msg {
                ServerMsg::PlayResponse { fen, color } => {
                    self.game.replace(Game {
                        board: ChessBoard::from_str(&fen).unwrap(),
                        color,
                    });
                    true
                }
                ServerMsg::InvalidSession => {
                    info!("received invalid session, going back to home");
                    ctx.link().navigator().unwrap().replace(&Route::Home);
                    true
                }
                ServerMsg::InvalidLobby => {
                    info!("received invalid lobby, going back to home");
                    ctx.link().navigator().unwrap().replace(&Route::Home);
                    true
                }
                ServerMsg::OpponentJoined => {
                    info!("opponent joined");
                    true
                }
                ServerMsg::PlayedMove(chess_move) => {
                    if let Some(game) = &mut self.game {
                        if let Err(err) = game.board.try_play(chess_move) {
                            warn!("tried to play invalid move {chess_move} ({err})");
                        };
                        true
                    } else {
                        false
                    }
                }
                other => {
                    info!("received {other:?}");
                    false
                }
            },
        }
    }
}
