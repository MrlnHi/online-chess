use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl From<chess::Color> for Color {
    fn from(value: chess::Color) -> Self {
        match value {
            chess::Color::White => Color::White,
            chess::Color::Black => Color::Black,
        }
    }
}

impl From<Color> for chess::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::White => chess::Color::White,
            Color::Black => chess::Color::Black,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Square(String);

impl From<chess::Square> for Square {
    fn from(value: chess::Square) -> Self {
        Self(value.to_string())
    }
}

impl TryFrom<Square> for chess::Square {
    type Error = chess::Error;

    fn try_from(value: Square) -> Result<Self, Self::Error> {
        value.0.parse()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Piece(String);

impl From<chess::Piece> for Piece {
    fn from(value: chess::Piece) -> Self {
        Self(<chess::Piece as ToString>::to_string(&value))
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
#[error("invalid piece {0}")]
pub struct InvalidPieceError(String);

impl TryFrom<Piece> for chess::Piece {
    type Error = InvalidPieceError;

    fn try_from(value: Piece) -> Result<Self, Self::Error> {
        match &value.0[..] {
            "p" => Ok(chess::Piece::Pawn),
            "n" => Ok(chess::Piece::Knight),
            "b" => Ok(chess::Piece::Bishop),
            "r" => Ok(chess::Piece::Rook),
            "q" => Ok(chess::Piece::Queen),
            "k" => Ok(chess::Piece::Knight),
            _ => Err(InvalidPieceError(value.0)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChessMove {
    source: Square,
    dest: Square,
    promotion: Option<Piece>,
}

impl From<chess::ChessMove> for ChessMove {
    fn from(value: chess::ChessMove) -> Self {
        Self {
            source: value.get_source().into(),
            dest: value.get_dest().into(),
            promotion: value.get_promotion().map(|piece| piece.into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum TryFromError {
    #[error("{0}")]
    ChessError(chess::Error),

    #[error(transparent)]
    InvalidPieceError(#[from] InvalidPieceError),
}

impl From<chess::Error> for TryFromError {
    fn from(value: chess::Error) -> Self {
        Self::ChessError(value)
    }
}

impl TryFrom<ChessMove> for chess::ChessMove {
    type Error = TryFromError;

    fn try_from(value: ChessMove) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.source.try_into()?,
            value.dest.try_into()?,
            match value.promotion {
                Some(piece) => Some(piece.try_into()?),
                None => None,
            },
        ))
    }
}
