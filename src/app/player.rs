use std::sync::{Arc, Weak};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::app::error::{ProcessError, SendError};
use crate::app::Room;
use crate::proto::s2c;

pub struct Player {
    session_id: Uuid,
    tx: UnboundedSender<Vec<u8>>,
    room: Option<Weak<RwLock<Room>>>,
}

impl Player {
    pub fn new(tx: UnboundedSender<Vec<u8>>) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            tx,
            room: None,
        }
    }

    pub fn get_session_id(&self) -> &Uuid {
        &self.session_id
    }

    pub fn get_room(&self) -> Option<Arc<RwLock<Room>>> {
        match &self.room {
            Some(w) => w.upgrade(),
            None => None,
        }
    }

    pub fn is_in_room(&self) -> bool {
        self.room.is_some()
    }

    pub async fn send(&self, msg: &s2c::Message<'_>) -> Result<(), SendError> {
        let mut buf = vec![];
        msg.encode(&mut buf).expect("TODO: panic message");

        self.tx.send(buf)?;
        Ok(())
    }

    pub async fn enter_room(&mut self, room: &Arc<RwLock<Room>>) -> Result<(), ProcessError> {
        if self.is_in_room() {
            return Err(ProcessError::InvalidOperation);
        }

        self.set_room_unchecked(room);

        self.send(&s2c::Message::RoomJoined {
            host_session_id: &room.read().await.get_host().read().await.get_session_id(),
        })
        .await?;

        println!(
            "Player {} joined room {:#02x?}",
            self.get_session_id(),
            room.read().await.get_code()
        );

        Ok(())
    }

    pub(crate) fn set_room_unchecked(&mut self, room: &Arc<RwLock<Room>>) {
        self.room = Some(Arc::downgrade(room));
    }
}
