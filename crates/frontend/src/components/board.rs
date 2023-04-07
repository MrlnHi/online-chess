use cozy_chess::{BitBoard, Board as ChessBoard, Color, File, Move, Piece, Rank, Square};
use log::info;
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub board: ChessBoard,
    pub color: Color,
    pub play_move: Callback<Move>,
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
                        if let [file, rank] = chars[..] {
                            let file = file.to_digit(10);
                            let rank = rank.to_digit(10);
                            match file.zip(rank) {
                                Some((file, rank)) => {
                                    let square = Square::new(
                                        File::index(file as usize),
                                        Rank::index(rank as usize),
                                    );
                                    link.send_message(Msg::ClickSquare(square));
                                }
                                None => {
                                    info!("invalid data-square value '{data}'");
                                }
                            }
                        } else {
                            info!("invalid data-square value '{data}'");
                        };
                    }
                })
            };
            let click_square = &click_square;
            (0..8)
            .flat_map(|rank| {
                (0..8).map(move |file| {
                    html! {
                        <div style={format!("background-color: {}; position: absolute; left: {}%; bottom: {}%; background-size: 100%; height: 12.5%; width: 12.5%;", if (file + rank) % 2 == 0 {"black"} else {"white"}, 12.5 * file as f32, 12.5 * rank as f32)}
                        data-square={format!("{file}{rank}")}
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
                        if let [file, rank] = chars[..] {
                            let file = file.to_digit(10);
                            let rank = rank.to_digit(10);
                            match file.zip(rank) {
                                Some((file, rank)) => {
                                    let square = Square::new(
                                        File::index(file as usize),
                                        Rank::index(rank as usize),
                                    );
                                    link.send_message(Msg::ClickPiece(square));
                                }
                                None => {
                                    info!("invalid data-square value '{data}'");
                                }
                            }
                        } else {
                            info!("invalid data-square value '{data}'");
                        };
                    }
                })
            };
            (0..8)
            .flat_map(|rank| {
                (0..8)
                    .map(move |file| {
                        Square::new(File::index(file), Rank::index(rank))
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
                        let style = format!("background-image: url('{url}'); position: absolute; left: {}%; bottom: {}%; background-size: 100%; height: 12.5%; width: 12.5%; cursor: grab; {}",
                            12.5 * square.file() as usize as f32,
                            12.5 * square.rank() as usize as f32,
                            if self.selected_square == Some(square) {
                                "background-color: rgba(255, 255, 0, 0.8)"
                            } else {
                                ""
                            },
                        );
                        html! {
                            <div style={style}
                            data-square={format!("{}{}", square.file() as usize, square.rank() as usize)}
                            onclick={click_piece.clone()}/>
                        }
                    })
            })
            .collect()
        };

        let moves: Option<Html> = self.selected_square.map(|selected_square| {
            let mut dest_squares = BitBoard::default();
            ctx.props()
                .board
                .generate_moves_for(selected_square.bitboard(), |moves| {
                    dest_squares |= moves.to;
                    false
                });
            dest_squares.into_iter().map(|dest| {
                let style = format!("position: absolute; left: {}%; bottom: {}%; height: 12.5%; width: 12.5%; background-color: #bbb; border-radius: 50%; pointer-events: none;",
                        12.5 * dest.file() as usize as f32,
                        12.5 * dest.rank() as usize as f32);
                    html! {
                        <div style={style}/>
                    }
            }).collect()
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
            Msg::ClickSquare(clicked) => {
                if let Some(selected) = self.selected_square.take() {
                    let chess_move = Move {
                        from: selected,
                        to: clicked,
                        promotion: None,
                    };
                    if ctx.props().board.is_legal(chess_move) {
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
