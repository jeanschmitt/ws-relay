use thiserror::Error;
use tokio::sync::mpsc;

use crate::proto;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("decode error")]
    Decode(#[from] proto::error::DecodeError),
    #[error("send error")]
    Send(#[from] SendError),
    #[error("invalid operation")]
    InvalidOperation,
    #[error("not in room")]
    NotInRoom,
    #[error("receiver not in room")]
    ReceiverNotInRoom,
    #[error("room not found")]
    RoomNotFound,
}

#[derive(Error, Debug)]
pub enum SendError {
    #[error("encode error")]
    Encode(#[from] proto::error::EncodeError),

    #[error("mpsc send error")]
    Send(#[from] mpsc::error::SendError<Vec<u8>>),
}
