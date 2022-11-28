use std::sync::Arc;

use tokio::sync::RwLock;

pub use app::App;
pub use player::Player;
pub use room::Room;

use crate::app::error::ProcessError;
use crate::proto::{c2s, s2c};

mod app;
pub mod error;
mod player;
mod room;

pub async fn process_message(
    sender: &Arc<RwLock<Player>>,
    app: &Arc<RwLock<App>>,
    msg: &[u8],
) -> Result<(), ProcessError> {
    let msg = c2s::Message::decode(msg)?;

    match msg {
        c2s::Message::SendToPlayer(mut fwd) => {
            let sender = sender.read().await;
            let sender_session_id = *sender.get_session_id();

            let receiver = match sender.get_room() {
                Some(room) => match room.read().await.get_player(&fwd.session_id) {
                    Some(player) => player,
                    None => return Err(ProcessError::PlayerNotFound),
                },
                None => return Err(ProcessError::NotInRoom),
            };

            println!(
                "Player {} sent {:?} to {}",
                sender.get_session_id(),
                &fwd.raw,
                receiver.read().await.get_session_id()
            );

            // In-place change session_id
            fwd.set_session_id(&sender_session_id);
            receiver
                .read()
                .await
                .send(&s2c::Message::ReceiveFromPlayer(fwd))
                .await?;

            Ok(())
        }
        c2s::Message::CreateRoom => app.write().await.create_room(&sender).await,
        c2s::Message::JoinRoom { code } => {
            let room = match app.read().await.get_room(code) {
                Some(room) => room.clone(),
                None => return Err(ProcessError::RoomNotFound),
            };

            room.write().await.add_player(sender).await;
            sender.write().await.enter_room(&room).await
        }
    }
}
