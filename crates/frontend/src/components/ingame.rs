use uuid::Uuid;
use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: Uuid,
}

#[function_component]
pub fn Ingame(props: &Props) -> Html {
    html! {
        <p>{format!("Ingame I guess ({})", props.id)}</p>
    }
}
