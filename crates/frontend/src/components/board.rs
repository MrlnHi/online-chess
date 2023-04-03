use chess::{Board as ChessBoard, ChessMove, Color, File, Piece, Rank, Square};
use log::info;
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub board: ChessBoard,
    pub color: Color,
    pub play_move: Callback<ChessMove>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Board {
    selected_square: Option<Square>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    SelectSquare(Square),
}

impl Component for Board {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let squares: Html = (0..8)
            .flat_map(|rank| {
                (0..8).map(move |file| {
                    html! {
                        <div style={format!("background-color: {}; position: absolute; bottom: {}%; left: {}%; background-size: 100%; height: 12.5%; width: 12.5%;", if (file + rank) % 2 == 0 {"black"} else {"white"}, 12.5 * rank as f32, 12.5 * file as f32)}
                        data-square={format!("{rank}{file}")}/>
                    }
                })
            })
            .collect();

        let click = {
            let link = ctx.link().clone();
            Callback::from(move |e: MouseEvent| {
                let link = link.clone();
                let element: HtmlElement = e.target_unchecked_into();
                if let Some(data) = element.get_attribute("data-square") {
                    let chars: Vec<_> = data.chars().collect();
                    match chars[..] {
                        [rank, file] => {
                            let rank = rank.to_digit(10);
                            let file = file.to_digit(10);
                            match rank.zip(file) {
                                Some((rank, file)) => {
                                    let square = Square::make_square(
                                        Rank::from_index(rank as usize),
                                        File::from_index(file as usize),
                                    );
                                    link.send_message(Msg::SelectSquare(square));
                                }
                                None => {
                                    info!("invalid data-square value '{data}'");
                                }
                            }
                        }
                        _ => info!("invalid data-square value '{data}'"),
                    };
                }
            })
        };

        let pieces: Html = (0..8)
            .flat_map(|rank| {
                (0..8)
                    .map(move |file| {
                        Square::make_square(Rank::from_index(rank), File::from_index(file))
                    })
                    .filter_map(|square| {
                        props
                            .board
                            .piece_on(square)
                            .zip(props.board.color_on(square))
                            .map(|(a, b)| (a, b, square))
                    })
                    .map(|(piece, color, square)| {
                        let url = format!(
                            "https://www.chess.com/chess-themes/pieces/neo/150/{}{}.png",
                            match color {
                                Color::White => "w",
                                Color::Black => "b",
                            },
                            match piece {
                                Piece::Pawn => "p",
                                Piece::Knight => "n",
                                Piece::Bishop => "b",
                                Piece::Rook => "r",
                                Piece::Queen => "q",
                                Piece::King => "k",
                            }
                        );
                        let style = format!("background-image: url('{url}'); position: absolute; bottom: {}%; left: {}%; background-size: 100%; height: 12.5%; width: 12.5%; cursor: grab; {}",
                            12.5 * square.get_rank().to_index() as f32,
                            12.5 * square.get_file().to_index() as f32,
                            if self.selected_square == Some(square) {
                                "background-color: rgba(255, 255, 0, 0.8)"
                            } else {""},
                        );
                        html! {
                            <div style={style}
                            data-square={format!("{}{}", square.get_rank().to_index(), square.get_file().to_index())}
                            onclick={click.clone()}/>
                        }
                    })
            })
            .collect();

        html! {
            <>
                <p>{"You are "}{if props.color == Color::White {"White"} else {"Black"}}</p>
                <div style="width: 600px; height: 600px; position: relative;">
                    <div style={"position: absolute; top: 0; left: 0; width: 100%; height: 100%;"}>
                        {squares}
                    </div>
                    <div style={"position: absolute; top: 0; left: 0; width: 100%; height: 100%;"}>
                        {pieces}
                    </div>
                </div>
            </>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SelectSquare(square) => {
                self.selected_square.replace(square);
                info!("selected {square}");
                true
            }
        }
    }
}
