use std::collections::HashMap;
use std::sync::{Arc, Weak};

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::app::Player;
use crate::code::Code;
use crate::proto::s2c;

pub struct Room {
    code: Code,
    host: Weak<RwLock<Player>>,
    players: HashMap<Uuid, Weak<RwLock<Player>>>,
}

impl Room {
    pub async fn new(host: &Arc<RwLock<Player>>) -> Self {
        let host_id = *host.read().await.get_session_id();
        let host = Arc::downgrade(host);

        Self {
            code: Code::new(),
            host: host.clone(),
            players: HashMap::from([(host_id, host)]),
        }
    }

    pub async fn add_player(&mut self, player: &Arc<RwLock<Player>>) {
        let session_id = *player.read().await.get_session_id();
        self.players.insert(session_id, Arc::downgrade(player));

        self.host
            .upgrade()
            .unwrap()
            .read()
            .await
            .send(&s2c::Message::PlayerJoined {
                player_session_id: &session_id,
            })
            .await
            .unwrap();
    }

    pub fn get_code(&self) -> &Code {
        &self.code
    }

    pub fn get_host(&self) -> Arc<RwLock<Player>> {
        self.host.upgrade().unwrap()
    }

    pub fn get_player(&self, session_id: &Uuid) -> Option<Arc<RwLock<Player>>> {
        self.players.get(session_id).and_then(|w| w.upgrade())
    }
}
