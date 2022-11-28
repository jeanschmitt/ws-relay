use bytes::{Buf, BufMut};

use crate::code::{self, Code};
use crate::proto::error::{DecodeError, EncodeError};
use crate::proto::ForwardMessage;

pub enum Message<'a> {
    SendToPlayer(ForwardMessage<'a>),
    CreateRoom,
    JoinRoom { code: &'a Code },
}

impl<'a> Message<'a> {
    pub fn encode<B>(&self, buf: &mut B) -> Result<(), EncodeError>
    where
        B: BufMut,
    {
        buf.put_u8(self.type_code());

        match self {
            Message::SendToPlayer(fwd) => fwd.encode(buf),
            Message::CreateRoom => Ok(()),
            Message::JoinRoom { code } => Ok(buf.put_slice(code.as_slice())),
        }
    }

    pub fn decode(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let remaining = buf.remaining();

        if remaining < 1 {
            return Err(DecodeError::BufferTooSmall { min: 1, remaining });
        }

        match buf[0] {
            1 => Ok(Message::SendToPlayer(ForwardMessage::decode(&buf[1..])?)),
            2 => Ok(Message::CreateRoom),
            3 => {
                let min = code::CODE_SIZE + 1;

                if remaining < min {
                    return Err(DecodeError::BufferTooSmall { min, remaining });
                }

                Ok(Message::JoinRoom {
                    code: buf[1..code::CODE_SIZE + 1].try_into().unwrap(),
                })
            }
            c => Err(DecodeError::BadMessageCode { code: c }),
        }
    }

    const fn type_code(&self) -> u8 {
        match self {
            Message::SendToPlayer { .. } => 1,
            Message::CreateRoom => 2,
            Message::JoinRoom { .. } => 3,
        }
    }
}
