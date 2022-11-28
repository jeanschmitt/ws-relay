use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::future::join_all;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::http::header::CONTENT_TYPE;
use warp::http::{HeaderValue, Response};
use warp::ws::{WebSocket, Ws};
use warp::{ws, Filter};

use crate::app;
use crate::app::error::ProcessError;
use crate::app::{App, Player};

const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");

pub async fn run(listen_addr: &SocketAddr, app: Arc<RwLock<App>>) {
    let app = warp::any().map(move || app.clone());

    let netcode = warp::path("netcode")
        .and(warp::ws())
        .and(app.clone())
        .map(|ws: Ws, app| ws.on_upgrade(move |socket| player_connected(socket, app)));

    let ping = warp::path("ping").map(|| {
        Response::builder()
            .header(CONTENT_TYPE, APPLICATION_JSON)
            .body("\"pong\"")
    });

    let list_players = warp::path("players")
        .and(app.clone())
        .and_then(list_players);
    let list_rooms = warp::path("rooms").and(app).and_then(list_rooms);

    let routes = ping.or(netcode).or(list_players).or(list_rooms);

    warp::serve(routes).run(*listen_addr).await;
}

async fn player_connected(ws: WebSocket, app: Arc<RwLock<App>>) {
    // Create the channel used to send S2C messages
    let (tx_s2c, rx_s2c) = mpsc::unbounded_channel::<Vec<u8>>();
    let mut rx_s2c = UnboundedReceiverStream::new(rx_s2c);

    // Create a Player associated with the connection
    let (player, player_id) = match app.write().await.add_player(Player::new(tx_s2c)).await {
        Ok(player) => {
            let session_id = *player.read().await.get_session_id();
            (player, session_id)
        }
        Err(e) => {
            eprintln!("failed to add player: {:?}", e);
            return;
        }
    };

    // Split the socket into a write half and a read half
    let (mut ws_tx, mut ws_rx) = ws.split();

    // Task forwarding messages from rx_s2c to ws_tx
    tokio::task::spawn(async move {
        while let Some(message) = rx_s2c.next().await {
            ws_tx
                .send(ws::Message::binary(message))
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Process incoming messages
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", &player_id, e);
                break;
            }
        };

        process_message(msg, &app, &player, &player_id)
            .await
            .unwrap_or_else(|e| eprintln!("process message error(id={}): {:?}", &player_id, e));
    }

    player_disconnected(&player_id, &app).await;
}

async fn process_message(
    msg: ws::Message,
    app: &Arc<RwLock<App>>,
    player: &Arc<RwLock<Player>>,
    player_id: &Uuid,
) -> Result<(), ProcessError> {
    if !msg.is_binary() {
        eprintln!("received non-binary message from {}", player_id);
        return Ok(());
    }

    app::process_message(player, app, msg.as_bytes()).await
}

async fn player_disconnected(session_id: &Uuid, app: &Arc<RwLock<App>>) {
    if let Err(e) = app.write().await.remove_player(session_id).await {
        eprintln!("failed to remove player: {:?}", e);
    }
}

async fn list_players(app: Arc<RwLock<App>>) -> Result<warp::reply::Json, Infallible> {
    Ok(warp::reply::json(
        &join_all(
            app.read()
                .await
                .get_players()
                .map(|player| async { format!("{}", player.read().await.get_session_id()) }),
        )
        .await,
    ))
}

async fn list_rooms(app: Arc<RwLock<App>>) -> Result<warp::reply::Json, Infallible> {
    Ok(warp::reply::json(
        &join_all(
            app.read()
                .await
                .get_rooms()
                .map(|room| async { format!("{:?}", room.read().await.get_code()) }),
        )
        .await,
    ))
}
