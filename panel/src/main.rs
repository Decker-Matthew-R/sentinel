mod connection_state;
mod framing;
mod link_server;
mod proto;

use connection_state::ConnectionState;
use link_server::LinkServer;
use std::time::Duration;
use tokio::sync::watch;

slint::include_modules!();

#[tokio::main]
async fn main() {
    let server = LinkServer::bind(9000).await.unwrap();
    let (tx, _rx) = watch::channel(ConnectionState::Waiting);

    tokio::spawn(async move {
        server.run(tx, Duration::from_secs(5)).await;
    });

    AppWindow::new().unwrap().run().unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn panel_tests_run() {}
}
