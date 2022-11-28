use bytes::BufMut;
use uuid::Uuid;

use crate::code::Code;
use crate::proto::error::{DecodeError, EncodeError};
use crate::proto::ForwardMessage;

pub enum Message<'a> {
    ReceiveFromPlayer(ForwardMessage<'a>),
    AssignSessionId { session_id: &'a Uuid },
    RoomCreated { code: &'a Code },
    RoomJoined { host_session_id: &'a Uuid },
    PlayerJoined { player_session_id: &'a Uuid },
}

impl<'a> Message<'a> {
    pub fn encode<B>(&self, buf: &mut B) -> Result<(), EncodeError>
    where
        B: BufMut,
    {
        buf.put_u8(self.type_code());

        match self {
            Message::ReceiveFromPlayer(fwd) => fwd.encode(buf),
            Message::AssignSessionId { session_id } => encode_uuid(buf, session_id),
            Message::RoomCreated { code } => Ok(buf.put_slice(code.as_slice())),
            Message::RoomJoined { host_session_id } => encode_uuid(buf, host_session_id),
            Message::PlayerJoined { player_session_id } => encode_uuid(buf, player_session_id),
        }
    }

    pub fn decode(buf: &'a [u8]) -> Result<Self, DecodeError> {
        //let remaining = buf.len();

        match buf[0] {
            1 => Ok(Message::ReceiveFromPlayer(ForwardMessage::decode(
                &buf[1..],
            )?)),
            2 => Ok(Message::AssignSessionId {
                session_id: decode_uuid(&buf[1..17])?,
            }),
            3 => Ok(Message::RoomCreated {
                code: buf[1..5].try_into().unwrap(),
            }),
            4 => Ok(Message::RoomJoined {
                host_session_id: decode_uuid(&buf[1..17])?,
            }),
            5 => Ok(Message::PlayerJoined {
                player_session_id: decode_uuid(&buf[1..17])?,
            }),
            c => Err(DecodeError::BadMessageCode { code: c }),
        }
    }

    const fn type_code(&self) -> u8 {
        match self {
            Message::ReceiveFromPlayer(_) => 1,
            Message::AssignSessionId { .. } => 2,
            Message::RoomCreated { .. } => 3,
            Message::RoomJoined { .. } => 4,
            Message::PlayerJoined { .. } => 5,
        }
    }
}

fn encode_uuid<B>(buf: &mut B, uuid: &Uuid) -> Result<(), EncodeError>
where
    B: BufMut,
{
    let remaining = buf.remaining_mut();
    if remaining < 16 {
        return Err(EncodeError::InsufficientCapacity {
            required: 16,
            remaining,
        });
    }

    buf.put_slice(uuid.as_bytes());
    Ok(())
}

fn decode_uuid(buf: &[u8]) -> Result<&Uuid, DecodeError> {
    let len = buf.len();
    if len != 16 {
        return Err(DecodeError::BufferTooSmall {
            remaining: len,
            min: 16,
        });
    }

    let b: &[u8; 16] = buf[0..16].try_into().unwrap();
    Ok(Uuid::from_bytes_ref(&b))
}
