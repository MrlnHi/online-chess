use chess::{Board as ChessBoard, ChessMove, Color};
use log::info;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub board: ChessBoard,
    pub color: Color,
    pub play_move: Callback<ChessMove>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Board {
    pub input_ref: NodeRef,
}

impl Component for Board {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let play_move = {
            let input = self.input_ref.clone();
            let play_move = props.play_move.clone();
            let board = props.board;
            Callback::from(move |_| {
                let input: HtmlInputElement = input.cast().unwrap();
                let input = input.value();
                match ChessMove::from_san(&board, &input) {
                    Ok(chess_move) => play_move.emit(chess_move),
                    Err(err) => info!("invalid move: {err}"),
                };
            })
        };

        html! {
            <>
                <p>{"You are "}{if props.color == Color::White {"White"} else {"Black"}}</p>
                <p>{props.board.to_string()}</p>
                <input placeholder={"Play move"} ref={&self.input_ref}/>
                <button onclick={play_move}>{"Play move"}</button>
            </>
        }
    }
}
