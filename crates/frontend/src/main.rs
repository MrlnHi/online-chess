use common::http::{HostResponse, JoinResponse};
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::{ingame::Ingame, menu::Menu};

mod components;

#[derive(Routable, PartialEq, Clone, Copy, Debug)]
enum Route {
    #[at("/")]
    Home,
    // TODO: Do not include session in url
    #[at("/game/:id/:session")]
    Ingame { id: Uuid, session: Uuid },
    #[at("/not-found")]
    #[not_found]
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Game {
    pub id: Uuid,
    pub session: Uuid,
}

impl From<HostResponse> for Game {
    fn from(value: HostResponse) -> Self {
        Self {
            id: value.lobby_id,
            session: value.session,
        }
    }
}

impl From<JoinResponse> for Game {
    fn from(value: JoinResponse) -> Self {
        Self {
            id: value.lobby_id,
            session: value.session,
        }
    }
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Menu /> },
        Route::Ingame { id, session } => html! { <Ingame {id} {session} /> },
        Route::NotFound => html! { "Not Found." },
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
