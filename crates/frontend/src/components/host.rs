use common::http::HostResponse;
use reqwasm::http::Request;
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Host {
    pub error: Option<String>,
}

pub enum Msg {
    Error(String),
    Join { id: Uuid, session: Uuid },
}

impl Component for Host {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        ctx.link().send_future(async {
            let response = match Request::post("/api/host").send().await {
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
                other => Msg::Error(format!(
                    "Unhandled status code {other} ({})",
                    response.status_text(),
                )),
            }
        });

        Host::default()
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        match &self.error {
            Some(err) => html! {
                <p>{"An error occured: "} {err}</p>
            },
            None => html! {
                <p>{"Hosting..."}</p>
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
                    .push(&Route::WaitingForOpponent { id, session });
                false
            }
        }
    }
}
