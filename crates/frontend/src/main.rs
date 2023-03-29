use common::{HostResponse, JoinResponse, Session};
use uuid::Uuid;
use yew::prelude::*;

use crate::component::{game::Game, menu::Menu};

mod component;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lobby {
    pub id: Uuid,
    pub session: Session,
}

impl From<HostResponse> for Lobby {
    fn from(value: HostResponse) -> Self {
        Self {
            id: value.lobby_id,
            session: value.session,
        }
    }
}

impl From<JoinResponse> for Lobby {
    fn from(value: JoinResponse) -> Self {
        Self {
            id: value.lobby_id,
            session: value.session,
        }
    }
}

#[function_component]
fn App() -> Html {
    let lobby = use_state(|| None);
    let on_game_start = {
        let game = lobby.clone();
        move |game_start| {
            game.set(Some(game_start));
        }
    };
    html! {
        if let Some(lobby) = *lobby {
            <Game {lobby}/>
        } else {
            <Menu {on_game_start}/>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
