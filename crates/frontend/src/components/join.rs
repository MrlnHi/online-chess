use common::http::HostResponse;
use reqwasm::http::Request;
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Join {
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Properties)]
pub struct Props {
    pub id: Uuid,
}

pub enum Msg {
    Error(String),
    Join { id: Uuid, session: Uuid },
}

impl Component for Join {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let id = ctx.props().id;
        ctx.link().send_future(async move {
            let response = match Request::post(&format!("/api/join/{id}")).send().await {
                Ok(response) => response,
                Err(err) => {
                    return Msg::Error(err.to_string());
                }
            };
            match response.status() {
                200 => {
                    let response: HostResponse = response.json().await.unwrap();
                    Msg::Join {
                        id: response.lobby_id,
                        session: response.session,
                    }
                }
                404 => Msg::Error("Unknown lobby".to_string()),
                409 => Msg::Error("Game is already running".to_string()),
                other => Msg::Error(format!(
                    "Unhandled status code {other} ({})",
                    response.status_text(),
                )),
            }
        });

        Self::default()
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        match &self.error {
            Some(err) => html! {
                <p>{"An error occured: "} {err}</p>
            },
            None => html! {
                <p>{"Joining..."}</p>
            },
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Error(err) => {
                self.error.replace(err);
                true
            }
            Msg::Join { id, session } => {
                ctx.link()
                    .navigator()
                    .unwrap()
                    .push(&Route::Ingame { id, session });
                false
            }
        }
    }
}
