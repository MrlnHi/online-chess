use std::str::FromStr;

use uuid::Uuid;
use web_sys::HtmlInputElement;
use yew::prelude::*;
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
            let navigator = ctx.link().navigator().unwrap();
            move |_| {
                navigator.push(&Route::Host);
            }
        };
        let join = {
            let input_ref = self.input_ref.clone();
            let link = ctx.link().clone();
            let navigator = link.navigator().unwrap();
            move |_| {
                let input: HtmlInputElement = input_ref.cast().unwrap();
                let input = input.value();
                let id = match Uuid::from_str(&input) {
                    Ok(id) => id,
                    Err(err) => {
                        link.send_message(Msg::Error(format!("Invalid lobby id '{input}': {err}")));
                        return;
                    }
                };
                navigator.push(&Route::Join { id });
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
