use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::app::error::{ProcessError, SendError};
use crate::app::{Player, Room};
use crate::code::Code;
use crate::proto::s2c;

pub struct App {
    players: HashMap<Uuid, Arc<RwLock<Player>>>,
    rooms: HashMap<Code, Arc<RwLock<Room>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            rooms: HashMap::new(),
        }
    }

    pub fn get_players(&self) -> impl Iterator<Item = &Arc<RwLock<Player>>> {
        self.players.values()
    }

    pub fn get_rooms(&self) -> impl Iterator<Item = &Arc<RwLock<Room>>> {
        self.rooms.values()
    }

    pub fn get_room(&self, code: &Code) -> Option<&Arc<RwLock<Room>>> {
        self.rooms.get(code)
    }

    pub async fn add_player(&mut self, player: Player) -> Result<Arc<RwLock<Player>>, SendError> {
        let session_id = *player.get_session_id();

        player
            .send(&s2c::Message::AssignSessionId {
                session_id: &session_id,
            })
            .await?;

        let player = Arc::new(RwLock::new(player));

        self.players.insert(session_id, player.clone());

        println!("Player {} joined", &session_id);

        Ok(player)
    }

    pub async fn remove_player(&mut self, session_id: &Uuid) -> Result<(), ProcessError> {
        // TODO: remove from Room, auto delete room etc
        self.players.remove(session_id);

        println!("Player {} left", session_id);

        Ok(())
    }

    pub async fn create_room(&mut self, host: &Arc<RwLock<Player>>) -> Result<(), ProcessError> {
        if host.read().await.is_in_room() {
            return Err(ProcessError::InvalidOperation);
        }

        let room = Room::new(host).await;
        let code = *room.get_code();

        let room = Arc::new(RwLock::new(room));

        {
            let mut host = host.write().await;
            host.send(&s2c::Message::RoomCreated { code: &code })
                .await?;
            host.set_room_unchecked(&room);
        }

        self.rooms.insert(code, room);

        println!("Room {} created", &code);

        Ok(())
    }
}
