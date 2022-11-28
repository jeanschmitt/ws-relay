use bytes::BufMut;
use uuid::Uuid;

use crate::proto::error::{DecodeError, EncodeError};

const UUID_LEN: usize = 16;

pub struct ForwardMessage<'a> {
    pub session_id: Uuid,
    pub raw: &'a [u8],
}

impl<'a> ForwardMessage<'a> {
    pub fn set_session_id(&mut self, session_id: &Uuid) {
        self.session_id = *session_id;
    }

    pub fn encode<B>(&self, buf: &mut B) -> Result<(), EncodeError>
    where
        B: BufMut,
    {
        let required = self.raw.len() + UUID_LEN;
        let remaining = buf.remaining_mut();
        if required > buf.remaining_mut() {
            return Err(EncodeError::InsufficientCapacity {
                required,
                remaining,
            });
        }

        self.encode_raw(buf);
        Ok(())
    }

    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: BufMut,
    {
        buf.put_slice(self.session_id.as_bytes());
        buf.put_slice(self.raw);
    }

    pub fn decode(buf: &'a [u8]) -> Result<Self, DecodeError> {
        const MIN_LEN: usize = UUID_LEN + 1;
        let remaining = buf.len();
        if remaining < MIN_LEN {
            return Err(DecodeError::BufferTooSmall {
                min: MIN_LEN,
                remaining,
            });
        }

        Ok(ForwardMessage {
            session_id: Uuid::from_slice(&buf[..UUID_LEN]).unwrap(), // buf len has already been checked
            raw: &buf[UUID_LEN..],
        })
    }
}
