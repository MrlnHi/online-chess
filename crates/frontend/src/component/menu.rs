use common::HostResponse;
use reqwasm::http::Request;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::Lobby;

#[derive(Debug, Default)]
pub struct Menu {
    input: String,
    output: String,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_game_start: Callback<Lobby>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Msg {
    Lobby(Lobby),
    OnInput(String),
    Error(String),
}

impl Component for Menu {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let host = {
            ctx.link().callback_future(|_| async {
                let response = match Request::post("/api/host").send().await {
                    Ok(response) => response,
                    Err(err) => return Msg::Error(err.to_string()),
                };
                match response.status() {
                    200 => {
                        let response: HostResponse = response.json().await.unwrap();
                        Msg::Lobby(response.into())
                    }
                    other => Msg::Error(format!("Unhandled status code {other}")),
                }
            })
        };
        let join = {
            let input = self.input.clone();
            ctx.link().callback_future(move |_| {
                let input = input.clone();
                async move {
                    let response = match Request::post(&format!("/api/join/{input}")).send().await {
                        Ok(response) => response,
                        Err(err) => return Msg::Error(err.to_string()),
                    };
                    match response.status() {
                        200 => {
                            let response: HostResponse = response.json().await.unwrap();
                            Msg::Lobby(response.into())
                        }
                        404 => Msg::Error("Unknown lobby".to_string()),
                        409 => Msg::Error("Game is already running".to_string()),
                        other => Msg::Error(format!(
                            "Unhandled status code {other}: {}",
                            response.text().await.unwrap_or_else(|_| "".to_string())
                        )),
                    }
                }
            })
        };
        let onchange = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::OnInput(input.value())
        });

        html! {
            <div>
                <button onclick={host}>{"Host Game"}</button>
                <div>
                    <input placeholder={"Lobby ID"} {onchange}/>
                    <button onclick={join}>{"Join Game"}</button>
                </div>
                <p>{&self.output}</p>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::Lobby(response) => ctx.props().on_game_start.emit(response),
            Msg::OnInput(input) => {
                self.input = input;
            }
            Msg::Error(err) => {
                self.output = err;
            }
        }
        true
    }
}
