use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use ws_relay::app::App;
use ws_relay::server;

const DEFAULT_PORT: u16 = 8080;
const PORT_ENV: &str = "PORT";

#[tokio::main]
async fn main() {
    let port = match env::var(PORT_ENV) {
        Ok(p) => p
            .parse::<u16>()
            .expect(format!("invalid PORT env \"{}\"", p).as_str()),
        Err(_) => DEFAULT_PORT,
    };

    let app = Arc::new(RwLock::new(App::new()));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    server::run(&addr, app).await;
}
