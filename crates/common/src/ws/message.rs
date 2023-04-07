use cozy_chess::{Color, Move, Piece, Square};
use std::io::{self, ErrorKind, Read, Write};
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

impl Message for Square {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let id = *self as usize;
        id.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let id = usize::decode(&mut read)?;
        Square::try_index(id).ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("square id ({} >= {})", id, Square::NUM),
            )
        })
    }
}

impl Message for Piece {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let id = *self as usize;
        id.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let id = usize::decode(&mut read)?;
        Piece::try_index(id)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("piece id {id}")))
    }
}

impl Message for Move {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        self.from.encode(&mut write)?;
        self.to.encode(&mut write)?;
        self.promotion.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let from = Square::decode(&mut read)?;
        let to = Square::decode(&mut read)?;
        let promotion = Option::decode(&mut read)?;

        Ok(Move {
            from,
            to,
            promotion,
        })
    }
}

impl Message for Color {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let id = *self as usize;
        id.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let id = usize::decode(&mut read)?;
        Color::try_index(id)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, format!("color id {id}")))
    }
}

impl Message for usize {
    fn encode(&self, mut write: impl Write) -> io::Result<()> {
        let a = *self as u64;
        a.encode(&mut write)?;
        write.flush()
    }

    fn decode(mut read: impl Read) -> io::Result<Self> {
        let a = u64::decode(&mut read)?;
        Ok(a as usize)
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
        let len = match usize::try_from(u64::decode(&mut read)?) {
            Ok(val) => val,
            Err(err) => return Err(io::Error::new(ErrorKind::InvalidData, err)),
        };
        let mut buf = vec![0; len];
        read.read_exact(&mut buf[..])?;
        String::from_utf8(buf).map_err(|err| io::Error::new(ErrorKind::InvalidData, err))
    }
}
