mod connection_state;
mod framing;
mod link_server;
mod proto;
mod state_store;

use connection_state::ConnectionState;
use link_server::LinkServer;
use state_store::StateStore;
use std::time::Duration;
use tokio::sync::watch;

slint::include_modules!();

fn format_metric(v: f32) -> String {
    format!("{:.1}", v)
}

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
    let (store, mut store_rx) = StateStore::new();

    tokio::spawn(async move {
        server.run(tx, &store, Duration::from_secs(5)).await;
    });

    let app = AppWindow::new().unwrap();
    let app_weak = app.as_weak();
    let app_weak2 = app.as_weak();

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

    tokio::spawn(async move {
        loop {
            if store_rx.changed().await.is_err() {
                break;
            }
            let snapshot = *store_rx.borrow();
            let handle = app_weak2.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = handle.upgrade() {
                    match snapshot {
                        Some(m) => {
                            if let Some(cpu) = m.cpu {
                                app.set_cpu_utilization(format_metric(cpu.utilization).into());
                                app.set_cpu_temperature(format_metric(cpu.temperature).into());
                            }
                            if let Some(gpu) = m.gpu {
                                app.set_gpu_utilization(format_metric(gpu.utilization).into());
                                app.set_gpu_temperature(format_metric(gpu.temperature).into());
                                app.set_gpu_hotspot_temperature(
                                    format_metric(gpu.hotspot_temperature).into(),
                                );
                                app.set_gpu_vram_used_gb(
                                    format_metric(gpu.vram_used_mb as f32 / 1024.0).into(),
                                );
                            }
                            if let Some(ram) = m.ram {
                                app.set_ram_used_gb(
                                    format_metric(ram.used_mb as f32 / 1024.0).into(),
                                );
                            }
                        }
                        None => {
                            app.set_cpu_utilization("--".into());
                            app.set_cpu_temperature("--".into());
                            app.set_gpu_utilization("--".into());
                            app.set_gpu_temperature("--".into());
                            app.set_gpu_hotspot_temperature("--".into());
                            app.set_gpu_vram_used_gb("--".into());
                            app.set_ram_used_gb("--".into());
                        }
                    }
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
    fn format_metric_one_decimal_place() {
        assert_eq!(format_metric(42.0), "42.0");
    }

    #[test]
    fn format_metric_rounds_correctly() {
        assert_eq!(format_metric(3.14159), "3.1");
    }

    #[test]
    fn ui_reflects_all_connection_states() {
        init_slint_once();
        let app = AppWindow::new().unwrap();

        assert_eq!(app.get_status(), ConnectionStatus::Waiting);
        assert_eq!(app.get_agent_version().as_str(), "");
        assert_eq!(app.get_uptime().as_str(), "");
        assert_eq!(app.get_cpu_utilization().as_str(), "--");
        assert_eq!(app.get_cpu_temperature().as_str(), "--");
        assert_eq!(app.get_gpu_utilization().as_str(), "--");
        assert_eq!(app.get_gpu_temperature().as_str(), "--");
        assert_eq!(app.get_gpu_hotspot_temperature().as_str(), "--");
        assert_eq!(app.get_gpu_vram_used_gb().as_str(), "--");
        assert_eq!(app.get_ram_used_gb().as_str(), "--");

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
