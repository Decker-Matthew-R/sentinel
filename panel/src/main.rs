mod connection_state;
mod framing;
mod link_server;
mod proto;
mod state_store;
mod view_models;

use connection_state::ConnectionState;
use link_server::LinkServer;
use state_store::StateStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

slint::include_modules!();

use view_models::{
    MetricState, cpu_temp_state, gpu_hotspot_state, gpu_temp_state, ram_usage_state,
    vram_usage_state,
};

fn to_tile_state(state: MetricState) -> TileState {
    match state {
        MetricState::Normal => TileState::Normal,
        MetricState::Warm => TileState::Warm,
        MetricState::Hot => TileState::Hot,
    }
}

fn format_metric(v: f32) -> String {
    format!("{:.1}", v)
}

fn format_fraction_gb(used_mb: u64, total_mb: u64) -> String {
    format!(
        "{:.1} / {:.1}",
        used_mb as f32 / 1024.0,
        total_mb as f32 / 1024.0
    )
}

fn format_utilization(used_mb: u64, total_mb: u64) -> String {
    if total_mb == 0 {
        return "--".to_string();
    }
    format!("{:.0}%", used_mb as f32 / total_mb as f32 * 100.0)
}

fn build_sparkline_path(values: &[f32]) -> String {
    if values.len() < 2 {
        return String::new();
    }
    let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let flat = (max - min).abs() < f32::EPSILON;
    let range = if flat { 1.0 } else { max - min };
    let n = values.len();
    let mut path = String::new();
    for (i, &v) in values.iter().enumerate() {
        let x = i as f32 / (n - 1) as f32 * 100.0;
        let y = if flat {
            50.0
        } else {
            100.0 - (v - min) / range * 100.0
        };
        if i == 0 {
            path.push_str(&format!("M {x:.1},{y:.1}"));
        } else {
            path.push_str(&format!(" L {x:.1},{y:.1}"));
        }
    }
    path
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
    let store = Arc::new(store);

    tokio::spawn({
        let store = Arc::clone(&store);
        async move {
            server.run(tx, &store, Duration::from_secs(5)).await;
        }
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

    tokio::spawn({
        let store = Arc::clone(&store);
        async move {
            loop {
                if store_rx.changed().await.is_err() {
                    break;
                }
                let snapshot = *store_rx.borrow();
                let (
                    cpu_util_spark,
                    cpu_temp_spark,
                    gpu_util_spark,
                    gpu_temp_spark,
                    gpu_hotspot_spark,
                    vram_spark,
                    ram_spark,
                ) = {
                    let h = store.histories();
                    (
                        build_sparkline_path(
                            &h.cpu_utilization
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                        ),
                        build_sparkline_path(
                            &h.cpu_temperature
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                        ),
                        build_sparkline_path(
                            &h.gpu_utilization
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                        ),
                        build_sparkline_path(
                            &h.gpu_temperature
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                        ),
                        build_sparkline_path(
                            &h.gpu_hotspot_temperature
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                        ),
                        build_sparkline_path(
                            &h.gpu_vram_used_mb
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                        ),
                        build_sparkline_path(
                            &h.ram_used_mb.values().iter().copied().collect::<Vec<_>>(),
                        ),
                    )
                };
                let handle = app_weak2.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = handle.upgrade() {
                        app.set_cpu_utilization_sparkline(cpu_util_spark.into());
                        app.set_cpu_temperature_sparkline(cpu_temp_spark.into());
                        app.set_gpu_utilization_sparkline(gpu_util_spark.into());
                        app.set_gpu_temperature_sparkline(gpu_temp_spark.into());
                        app.set_gpu_hotspot_sparkline(gpu_hotspot_spark.into());
                        app.set_gpu_vram_sparkline(vram_spark.into());
                        app.set_ram_sparkline(ram_spark.into());
                        match snapshot {
                            Some(m) => {
                                if let Some(cpu) = m.cpu {
                                    app.set_cpu_utilization(format_metric(cpu.utilization).into());
                                    app.set_cpu_temperature(format_metric(cpu.temperature).into());
                                    app.set_cpu_temperature_state(to_tile_state(cpu_temp_state(
                                        cpu.temperature,
                                    )));
                                }
                                if let Some(gpu) = m.gpu {
                                    app.set_gpu_utilization(format_metric(gpu.utilization).into());
                                    app.set_gpu_temperature(format_metric(gpu.temperature).into());
                                    app.set_gpu_temperature_state(to_tile_state(gpu_temp_state(
                                        gpu.temperature,
                                    )));
                                    app.set_gpu_hotspot_temperature(
                                        format_metric(gpu.hotspot_temperature).into(),
                                    );
                                    app.set_gpu_hotspot_state(to_tile_state(gpu_hotspot_state(
                                        gpu.hotspot_temperature,
                                    )));
                                    app.set_gpu_vram_used_gb(
                                        format_fraction_gb(gpu.vram_used_mb, gpu.vram_total_mb)
                                            .into(),
                                    );
                                    app.set_vram_utilization(
                                        format_utilization(gpu.vram_used_mb, gpu.vram_total_mb)
                                            .into(),
                                    );
                                    app.set_vram_state(to_tile_state(vram_usage_state(
                                        gpu.vram_used_mb,
                                        gpu.vram_total_mb,
                                    )));
                                }
                                if let Some(ram) = m.ram {
                                    app.set_ram_used_gb(
                                        format_fraction_gb(ram.used_mb, ram.total_mb).into(),
                                    );
                                    app.set_ram_utilization(
                                        format_utilization(ram.used_mb, ram.total_mb).into(),
                                    );
                                    app.set_ram_state(to_tile_state(ram_usage_state(
                                        ram.used_mb,
                                        ram.total_mb,
                                    )));
                                }
                            }
                            None => {
                                app.set_cpu_utilization("--".into());
                                app.set_cpu_temperature("--".into());
                                app.set_cpu_temperature_state(TileState::Normal);
                                app.set_gpu_utilization("--".into());
                                app.set_gpu_temperature("--".into());
                                app.set_gpu_temperature_state(TileState::Normal);
                                app.set_gpu_hotspot_temperature("--".into());
                                app.set_gpu_hotspot_state(TileState::Normal);
                                app.set_gpu_vram_used_gb("--".into());
                                app.set_vram_utilization("--".into());
                                app.set_vram_state(TileState::Normal);
                                app.set_ram_used_gb("--".into());
                                app.set_ram_utilization("--".into());
                                app.set_ram_state(TileState::Normal);
                            }
                        }
                    }
                });
            }
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
    fn build_sparkline_empty_slice_returns_empty() {
        assert_eq!(build_sparkline_path(&[]), "");
    }

    #[test]
    fn build_sparkline_single_point_returns_empty() {
        assert_eq!(build_sparkline_path(&[42.0]), "");
    }

    #[test]
    fn build_sparkline_flat_line_uses_midpoint_y() {
        let path = build_sparkline_path(&[5.0, 5.0, 5.0]);
        assert!(path.starts_with("M 0.0,50.0"), "got: {path}");
        assert!(path.ends_with("L 100.0,50.0"), "got: {path}");
    }

    #[test]
    fn build_sparkline_ascending_maps_to_descending_y() {
        assert_eq!(
            build_sparkline_path(&[0.0, 100.0]),
            "M 0.0,100.0 L 100.0,0.0"
        );
    }

    #[test]
    fn format_metric_one_decimal_place() {
        assert_eq!(format_metric(42.0), "42.0");
    }

    #[test]
    fn format_metric_rounds_correctly() {
        assert_eq!(format_metric(3.14159), "3.1");
    }

    #[test]
    fn format_fraction_gb_shows_used_and_total() {
        assert_eq!(format_fraction_gb(8704, 32768), "8.5 / 32.0");
    }

    #[test]
    fn format_utilization_shows_integer_percent() {
        assert_eq!(format_utilization(7500, 10000), "75%");
    }

    #[test]
    fn format_utilization_zero_total_returns_placeholder() {
        assert_eq!(format_utilization(0, 0), "--");
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
        assert_eq!(app.get_vram_utilization().as_str(), "--");
        assert_eq!(app.get_ram_used_gb().as_str(), "--");
        assert_eq!(app.get_ram_utilization().as_str(), "--");

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
