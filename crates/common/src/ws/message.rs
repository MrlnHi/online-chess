use std::io::{self, ErrorKind, Read, Write};

use chess::{ChessMove, Color, Piece, Square};
use uuid::Uuid;

#[cfg(feature = "reqwasm")]
use reqwasm::websocket::Message as ReqwasmMessage;

#[cfg(feature = "axum")]
use axum::extract::ws::Message as AxumMessage;

pub trait Message: Sized {
    fn encode(&self, write: impl Write) -> io::Result<()>;

    fn decode(read: impl Read) -> io::Result<Self>;

    #[cfg(feature = "axum")]
    fn to_axum_message(&self) -> io::Result<AxumMessage> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(AxumMessage::Binary(buf))
    }

    #[cfg(feature = "axum")]
    fn from_axum_message(msg: AxumMessage) -> io::Result<Self> {
        match msg {
            AxumMessage::Binary(bytes) => Self::decode(&bytes[..]),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid message type of {msg:?}"),
            )),
        }
    }

    #[cfg(feature = "reqwasm")]
    fn to_reqwasm_message(&self) -> io::Result<ReqwasmMessage> {
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(ReqwasmMessage::Bytes(buf))
    }

    #[cfg(feature = "reqwasm")]
    fn from_reqwasm_message(msg: ReqwasmMessage) -> io::Result<Self> {
        match msg {
            ReqwasmMessage::Bytes(bytes) => Self::decode(&bytes[..]),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid message type of {msg:?}"),
            )),
        }
    }
}

impl Message for Uuid {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        write.write_all(self.as_bytes())?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let mut buf = [0; 16];
        read.read_exact(&mut buf)?;
        Ok(Uuid::from_bytes(buf))
    }
}

impl Message for Square {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        write.write_all(&[self.to_int()])?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let mut buf = [0];
        read.read_exact(&mut buf)?;
        let [square] = buf;
        if square as usize >= chess::NUM_SQUARES {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("invalid square ({} >= {})", square, chess::NUM_SQUARES),
            ));
        }
        Ok(unsafe { Square::new(square) })
    }
}

impl Message for u8 {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        write.write_all(&[*self])?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let mut buf = [0];
        read.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Message for Piece {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let id = match self {
            Piece::Pawn => 0u8,
            Piece::Knight => 1u8,
            Piece::Bishop => 2u8,
            Piece::Rook => 3u8,
            Piece::Queen => 4u8,
            Piece::King => 5u8,
        };
        id.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let id = u8::decode(&mut read)?;
        match id {
            0 => Ok(Piece::Pawn),
            1 => Ok(Piece::Knight),
            2 => Ok(Piece::Bishop),
            3 => Ok(Piece::Rook),
            4 => Ok(Piece::Queen),
            5 => Ok(Piece::King),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("piece id {id}"),
            )),
        }
    }
}

impl<T: Message> Message for Option<T> {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        match self {
            Some(val) => {
                write.write_all(&[0])?;
                val.encode(&mut write)?;
            }
            None => {
                write.write_all(&[1])?;
            }
        }
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let id = u8::decode(&mut read)?;
        match id {
            0 => Ok(Some(T::decode(&mut read)?)),
            1 => Ok(None),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("option id {id}"),
            )),
        }
    }
}

impl Message for ChessMove {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        self.get_source().encode(&mut write)?;
        self.get_dest().encode(&mut write)?;
        self.get_promotion().encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let source = Square::decode(&mut read)?;
        let dest = Square::decode(&mut read)?;
        let promotion = Option::decode(&mut read)?;

        Ok(ChessMove::new(source, dest, promotion))
    }
}

impl Message for Color {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let id = match self {
            Color::White => 0u8,
            Color::Black => 1u8,
        };
        id.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let id = u8::decode(&mut read)?;
        match id {
            0 => Ok(Color::White),
            1 => Ok(Color::Black),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("color id {id}"),
            )),
        }
    }
}

impl Message for u64 {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        write.write_all(&self.to_le_bytes())?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let mut buf = [0; 8];
        read.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl Message for String {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let len = self.len() as u64;
        len.encode(&mut write)?;
        write.write_all(self.as_bytes())?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let len = u64::decode(&mut read)? as usize;
        let mut buf = vec![0; len];
        read.read_exact(&mut buf[..])?;
        String::from_utf8(buf).map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
    }
}
