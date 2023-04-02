use common::http::HostResponse;
use reqwasm::http::Request;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};
use yew_router::scope_ext::RouterScopeExt;

use crate::Route;

#[derive(Debug, Default)]
pub struct Menu {
    input_ref: NodeRef,
    output: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Error(String),
}

impl Component for Menu {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let host = {
            let link = ctx.link().clone();
            move |_| {
                let link = link.clone();
                spawn_local(async move {
                    let response = match Request::post("/api/host").send().await {
                        Ok(response) => response,
                        Err(err) => {
                            link.send_message(Msg::Error(err.to_string()));
                            return;
                        }
                    };
                    match response.status() {
                        200 => {
                            let response: HostResponse = response.json().await.unwrap();
                            link.navigator().unwrap().push(&Route::Ingame {
                                id: response.lobby_id,
                                session: response.session,
                            });
                        }
                        other => {
                            link.send_message(Msg::Error(format!("Unhandled status code {other}")))
                        }
                    }
                })
            }
        };
        let join = {
            let input_ref = self.input_ref.clone();
            let link = ctx.link().clone();
            move |_| {
                let input: HtmlInputElement = input_ref.cast().unwrap();
                let input = input.value();
                let link = link.clone();
                spawn_local(async move {
                    let response = match Request::post(&format!("/api/join/{input}")).send().await {
                        Ok(response) => response,
                        Err(err) => {
                            link.send_message(Msg::Error(err.to_string()));
                            return;
                        }
                    };
                    match response.status() {
                        200 => {
                            let response: HostResponse = response.json().await.unwrap();
                            link.navigator().unwrap().push(&Route::Ingame {
                                id: response.lobby_id,
                                session: response.session,
                            });
                        }
                        404 => link.send_message(Msg::Error("Unknown lobby".to_string())),
                        409 => link.send_message(Msg::Error("Game is already running".to_string())),
                        other => {
                            link.send_message(Msg::Error(format!("Unhandled status code {other}")))
                        }
                    }
                })
            }
        };

        html! {
            <div>
                <button onclick={host}>{"Host Game"}</button>
                <div>
                    <input placeholder={"Lobby ID"} ref={&self.input_ref}/>
                    <button onclick={join}>{"Join Game"}</button>
                </div>
                <p>{&self.output}</p>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::Error(err) => {
                self.output = err;
            }
        }
        true
    }
}
