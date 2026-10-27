mod connection_state;
mod framing;
mod link_server;
mod proto;

use connection_state::ConnectionState;
use link_server::LinkServer;
use std::time::Duration;
use tokio::sync::watch;

slint::include_modules!();

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m {secs}s")
    } else if mins > 0 {
        format!("{mins}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

fn to_slint_props(state: &ConnectionState) -> (ConnectionStatus, String, String) {
    match state {
        ConnectionState::Waiting => (ConnectionStatus::Waiting, String::new(), String::new()),
        ConnectionState::Connected {
            agent_version,
            uptime_seconds,
        } => (
            ConnectionStatus::Connected,
            agent_version.clone(),
            format_uptime(*uptime_seconds),
        ),
        ConnectionState::Stale { agent_version } => (
            ConnectionStatus::Stale,
            agent_version.clone(),
            String::new(),
        ),
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
            let (status, version, uptime) = to_slint_props(&state);
            let handle = app_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = handle.upgrade() {
                    app.set_status(status);
                    app.set_agent_version(version.into());
                    app.set_uptime(uptime.into());
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
        assert_eq!(app.get_uptime().as_str(), "");

        let (status, version, uptime) = to_slint_props(&ConnectionState::Connected {
            agent_version: "fake-1.0.0".into(),
            uptime_seconds: 3661,
        });
        app.set_status(status);
        app.set_agent_version(version.into());
        app.set_uptime(uptime.into());
        assert_eq!(app.get_status(), ConnectionStatus::Connected);
        assert_eq!(app.get_agent_version().as_str(), "fake-1.0.0");
        assert_eq!(app.get_uptime().as_str(), "1h 1m 1s");

        let (status, version, uptime) = to_slint_props(&ConnectionState::Stale {
            agent_version: "fake-1.0.0".into(),
        });
        app.set_status(status);
        app.set_agent_version(version.into());
        app.set_uptime(uptime.into());
        assert_eq!(app.get_status(), ConnectionStatus::Stale);
        assert_eq!(app.get_agent_version().as_str(), "fake-1.0.0");
        assert_eq!(app.get_uptime().as_str(), "");
    }

    #[test]
    fn waiting_state_maps_to_waiting_status() {
        let (status, version, uptime) = to_slint_props(&ConnectionState::Waiting);
        assert_eq!(status, ConnectionStatus::Waiting);
        assert!(version.is_empty());
        assert!(uptime.is_empty());
    }

    #[test]
    fn connected_state_maps_to_connected_status() {
        let (status, version, uptime) = to_slint_props(&ConnectionState::Connected {
            agent_version: "0.1.0".into(),
            uptime_seconds: 3661,
        });
        assert_eq!(status, ConnectionStatus::Connected);
        assert_eq!(version, "0.1.0");
        assert_eq!(uptime, "1h 1m 1s");
    }

    #[test]
    fn stale_state_maps_to_stale_status() {
        let (status, version, uptime) = to_slint_props(&ConnectionState::Stale {
            agent_version: "0.1.0".into(),
        });
        assert_eq!(status, ConnectionStatus::Stale);
        assert_eq!(version, "0.1.0");
        assert!(uptime.is_empty());
    }

    #[test]
    fn format_uptime_seconds_only() {
        assert_eq!(format_uptime(45), "45s");
    }

    #[test]
    fn format_uptime_minutes_and_seconds() {
        assert_eq!(format_uptime(125), "2m 5s");
    }

    #[test]
    fn format_uptime_hours_minutes_seconds() {
        assert_eq!(format_uptime(3661), "1h 1m 1s");
    }

    #[test]
    fn format_uptime_days_drops_seconds() {
        assert_eq!(format_uptime(90061), "1d 1h 1m");
    }
}
