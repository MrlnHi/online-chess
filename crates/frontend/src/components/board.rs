use chess::{Board as ChessBoard, ChessMove, Color, File, MoveGen, Piece, Rank, Square};
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
    ClickPiece(Square),
    ClickSquare(Square),
}

impl Component for Board {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let squares: Html = {
            let click_square = {
                let link = ctx.link().clone();
                Callback::from(move |e: MouseEvent| {
                    info!("click square");
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
                                        link.send_message(Msg::ClickSquare(square));
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
            let click_square = &click_square;
            (0..8)
            .flat_map(|rank| {
                (0..8).map(move |file| {
                    html! {
                        <div style={format!("background-color: {}; position: absolute; bottom: {}%; left: {}%; background-size: 100%; height: 12.5%; width: 12.5%;", if (file + rank) % 2 == 0 {"black"} else {"white"}, 12.5 * rank as f32, 12.5 * file as f32)}
                        data-square={format!("{rank}{file}")}
                        onclick={click_square.clone()}/>
                    }
                })
            })
            .collect()
        };

        let pieces: Html = {
            let click_piece = {
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
                                        link.send_message(Msg::ClickPiece(square));
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
            (0..8)
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
                            onclick={click_piece.clone()}/>
                        }
                    })
            })
            .collect()
        };

        let moves: Option<Html> = self.selected_square.map(|selected_square| {
            MoveGen::new_legal(&ctx.props().board)
                .filter(|chess_move| chess_move.get_source() == selected_square)
                .map(|chess_move| {
                    let style = format!("position: absolute; bottom: {}%; left: {}%; height: 12.5%; width: 12.5%; background-color: #bbb; border-radius: 50%; pointer-events: none;",
                        12.5 * chess_move.get_dest().get_rank().to_index() as f32, 
                        12.5 * chess_move.get_dest().get_file().to_index() as f32);
                    html! {
                        <div style={style}/>
                    }
                })
                .collect()
        });

        html! {
            <>
                <p>{"You are "}{if props.color == Color::White {"White"} else {"Black"}}</p>
                <div style="width: 600px; height: 600px; position: relative;">
                    {squares}
                    {pieces}
                    {moves}
                </div>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ClickPiece(square) => match ctx.props().board.color_on(square) {
                Some(color) if color == ctx.props().color => {
                    self.selected_square.replace(square);
                    true
                }
                _ => {
                    ctx.link().send_message(Msg::ClickSquare(square));
                    false
                }
            },
            Msg::ClickSquare(square) => {
                if let Some(selected) = self.selected_square.take() {
                    let chess_move = ChessMove::new(selected, square, None);
                    if ctx.props().board.legal(chess_move) {
                        ctx.props().play_move.emit(chess_move);
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
}
