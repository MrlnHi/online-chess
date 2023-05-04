use crate::components::{
    host::Host, ingame::Ingame, join::Join, menu::Menu, waiting_for_opponent::WaitingForOpponent,
};
use common::http::{HostResponse, JoinResponse};
use uuid::Uuid;
use yew::prelude::*;
use yew_router::prelude::*;

mod components;

#[derive(Routable, PartialEq, Clone, Copy, Debug)]
enum Route {
    #[at("/")]
    Home,
    #[at("/host")]
    Host,
    // TODO: Do not include session in url
    #[at("/waiting-for-opponent/:id/:session")]
    WaitingForOpponent { id: Uuid, session: Uuid },
    #[at("/join/:id")]
    Join { id: Uuid },
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
        Route::Host => html! { <Host /> },
        Route::WaitingForOpponent { id, session } => {
            html! { <WaitingForOpponent {id} {session} /> }
        }
        Route::Join { id } => html! { <Join {id} /> },
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
