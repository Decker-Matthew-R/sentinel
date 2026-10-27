mod connection_state;
mod framing;
mod link_server;
mod proto;

use connection_state::ConnectionState;
use link_server::LinkServer;
use std::time::Duration;
use tokio::sync::watch;

slint::include_modules!();

fn to_slint_props(state: &ConnectionState) -> (ConnectionStatus, String) {
    match state {
        ConnectionState::Waiting => (ConnectionStatus::Waiting, String::new()),
        ConnectionState::Connected { agent_version } => {
            (ConnectionStatus::Connected, agent_version.clone())
        }
        ConnectionState::Stale { agent_version } => {
            (ConnectionStatus::Stale, agent_version.clone())
        }
    }
}

#[tokio::main]
async fn main() {
    let server = LinkServer::bind(9000).await.unwrap();
    let (tx, rx) = watch::channel(ConnectionState::Waiting);

    tokio::spawn(async move {
        server.run(tx, Duration::from_secs(5)).await;
    });

    let app = AppWindow::new().unwrap();
    let app_weak = app.as_weak();

    tokio::spawn(async move {
        let mut rx = rx;
        loop {
            if rx.changed().await.is_err() {
                break;
            }
            let state = rx.borrow().clone();
            let (status, version) = to_slint_props(&state);
            let handle = app_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = handle.upgrade() {
                    app.set_status(status);
                    app.set_agent_version(version.into());
                }
            });
        }
    });

    app.run().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn init_slint_once() {
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            slint::platform::set_platform(Box::new(i_slint_backend_testing::TestingBackend::new(
                i_slint_backend_testing::TestingBackendOptions::default(),
            )))
            .unwrap();
        });
    }

    #[test]
    fn panel_tests_run() {}

    #[test]
    fn ui_reflects_all_connection_states() {
        init_slint_once();
        let app = AppWindow::new().unwrap();

        assert_eq!(app.get_status(), ConnectionStatus::Waiting);
        assert_eq!(app.get_agent_version().as_str(), "");

        let (status, version) = to_slint_props(&ConnectionState::Connected {
            agent_version: "fake-1.0.0".into(),
        });
        app.set_status(status);
        app.set_agent_version(version.into());
        assert_eq!(app.get_status(), ConnectionStatus::Connected);
        assert_eq!(app.get_agent_version().as_str(), "fake-1.0.0");

        let (status, version) = to_slint_props(&ConnectionState::Stale {
            agent_version: "fake-1.0.0".into(),
        });
        app.set_status(status);
        app.set_agent_version(version.into());
        assert_eq!(app.get_status(), ConnectionStatus::Stale);
        assert_eq!(app.get_agent_version().as_str(), "fake-1.0.0");
    }

    #[test]
    fn waiting_state_maps_to_waiting_status() {
        let (status, version) = to_slint_props(&ConnectionState::Waiting);
        assert_eq!(status, ConnectionStatus::Waiting);
        assert!(version.is_empty());
    }

    #[test]
    fn connected_state_maps_to_connected_status() {
        let (status, version) = to_slint_props(&ConnectionState::Connected {
            agent_version: "0.1.0".into(),
        });
        assert_eq!(status, ConnectionStatus::Connected);
        assert_eq!(version, "0.1.0");
    }

    #[test]
    fn stale_state_maps_to_stale_status() {
        let (status, version) = to_slint_props(&ConnectionState::Stale {
            agent_version: "0.1.0".into(),
        });
        assert_eq!(status, ConnectionStatus::Stale);
        assert_eq!(version, "0.1.0");
    }
}
