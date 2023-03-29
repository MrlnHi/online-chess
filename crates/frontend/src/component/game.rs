use yew::{function_component, html, Html, Properties};

use crate::Lobby;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub lobby: Lobby,
}

#[function_component]
pub fn Game(props: &Props) -> Html {
    html! {
        <p>{format!("{:?}", props.lobby)}</p>
    }
}
