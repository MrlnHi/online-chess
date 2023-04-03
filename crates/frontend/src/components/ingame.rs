use std::str::FromStr;

use crate::Route;

use super::board::Board;

use chess::{Board as ChessBoard, ChessMove, Color};
use common::ws::{ClientMsg, ServerMsg};
use futures::{
    channel::mpsc::{channel, Sender},
    SinkExt, StreamExt,
};
use log::info;
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
                    .into(),
                )
                .await
                .unwrap();
                info!("sent PlayRequest");
                spawn_local(async move {
                    while let Some(msg) = receive.next().await {
                        match msg {
                            Ok(msg) => match msg.try_into() {
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
                        send.send(msg.into()).await.unwrap();
                    }
                });
            });
        }

        Self { tx, game: None }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> Html {
        if let Some(Game { board, color }) = self.game {
            let play_move = {
                let tx = self.tx.clone();
                Callback::from(move |chess_move: ChessMove| {
                    let mut tx = tx.clone();
                    spawn_local(async move {
                        tx.send(ClientMsg::PlayMove(chess_move.into()))
                            .await
                            .unwrap();
                    })
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
                        color: color.into(),
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
                        // TODO: Error handling
                        game.board = game.board.make_move_new(chess_move.try_into().unwrap());
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
