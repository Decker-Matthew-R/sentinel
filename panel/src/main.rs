mod framing;
mod link_server;
mod proto;

use link_server::LinkServer;

slint::include_modules!();

#[tokio::main]
async fn main() {
    let server = LinkServer::bind(9000).await.unwrap();

    tokio::spawn(async move {
        loop {
            let _ = server.accept_hello().await;
        }
    });

    AppWindow::new().unwrap().run().unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn panel_tests_run() {}
}
