mod connection_state;
mod framing;
mod link_server;
mod proto;
mod state_store;
mod view_models;

use crate::proto::sentinel::v1::MetricsSnapshot;
use connection_state::ConnectionState;
use link_server::LinkServer;
use state_store::StateStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

slint::include_modules!();

use view_models::{
    MetricState, cpu_temp_state, cpu_util_state, gpu_hotspot_state, gpu_temp_state, gpu_util_state,
    ram_usage_state, vram_usage_state,
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

fn update_display_totals(
    snapshot: Option<MetricsSnapshot>,
    last_vram: &mut u64,
    last_ram: &mut u64,
) {
    match snapshot {
        Some(m) => {
            if let Some(gpu) = m.gpu {
                *last_vram = gpu.vram_total_mb;
            }
            if let Some(ram) = m.ram {
                *last_ram = ram.total_mb;
            }
        }
        None => {
            *last_vram = 0;
            *last_ram = 0;
        }
    }
}

fn build_colored_sparkline_paths(
    values: &[f32],
    state_fn: impl Fn(f32) -> MetricState,
) -> (String, String, String) {
    if values.len() < 2 {
        return (String::new(), String::new(), String::new());
    }
    let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let flat = (max - min).abs() < f32::EPSILON;
    let range = if flat { 1.0 } else { max - min };
    let n = values.len();
    let mut cool = String::new();
    let mut warm = String::new();
    let mut hot = String::new();
    let mut prev_state: Option<MetricState> = None;
    for (i, window) in values.windows(2).enumerate() {
        let px = i as f32 / (n - 1) as f32 * 100.0;
        let py = if flat {
            50.0
        } else {
            100.0 - (window[0] - min) / range * 100.0
        };
        let cx = (i + 1) as f32 / (n - 1) as f32 * 100.0;
        let cy = if flat {
            50.0
        } else {
            100.0 - (window[1] - min) / range * 100.0
        };
        let state = state_fn(window[1]);
        let path = match state {
            MetricState::Normal => &mut cool,
            MetricState::Warm => &mut warm,
            MetricState::Hot => &mut hot,
        };
        if prev_state.as_ref() != Some(&state) {
            if !path.is_empty() {
                path.push(' ');
            }
            path.push_str(&format!("M {px:.1},{py:.1} L {cx:.1},{cy:.1}"));
        } else {
            path.push_str(&format!(" L {cx:.1},{cy:.1}"));
        }
        prev_state = Some(state);
    }
    (cool, warm, hot)
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
            let mut last_vram_total_mb: u64 = 0;
            let mut last_ram_total_mb: u64 = 0;
            loop {
                if store_rx.changed().await.is_err() {
                    break;
                }
                let snapshot = *store_rx.borrow();
                update_display_totals(snapshot, &mut last_vram_total_mb, &mut last_ram_total_mb);
                let vram_total_mb = last_vram_total_mb;
                let ram_total_mb = last_ram_total_mb;
                let (
                    (cpu_util_cool, cpu_util_warm, cpu_util_hot),
                    (cpu_temp_cool, cpu_temp_warm, cpu_temp_hot),
                    (gpu_util_cool, gpu_util_warm, gpu_util_hot),
                    (gpu_temp_cool, gpu_temp_warm, gpu_temp_hot),
                    (gpu_hotspot_cool, gpu_hotspot_warm, gpu_hotspot_hot),
                    (vram_cool, vram_warm, vram_hot),
                    (ram_cool, ram_warm, ram_hot),
                ) = {
                    let h = store.histories();
                    (
                        build_colored_sparkline_paths(
                            &h.cpu_utilization
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                            cpu_util_state,
                        ),
                        build_colored_sparkline_paths(
                            &h.cpu_temperature
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                            cpu_temp_state,
                        ),
                        build_colored_sparkline_paths(
                            &h.gpu_utilization
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                            gpu_util_state,
                        ),
                        build_colored_sparkline_paths(
                            &h.gpu_temperature
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                            gpu_temp_state,
                        ),
                        build_colored_sparkline_paths(
                            &h.gpu_hotspot_temperature
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                            gpu_hotspot_state,
                        ),
                        build_colored_sparkline_paths(
                            &h.gpu_vram_used_mb
                                .values()
                                .iter()
                                .copied()
                                .collect::<Vec<_>>(),
                            |v| vram_usage_state(v as u64, vram_total_mb),
                        ),
                        build_colored_sparkline_paths(
                            &h.ram_used_mb.values().iter().copied().collect::<Vec<_>>(),
                            |v| ram_usage_state(v as u64, ram_total_mb),
                        ),
                    )
                };
                let handle = app_weak2.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = handle.upgrade() {
                        app.set_cpu_utilization_cool(cpu_util_cool.into());
                        app.set_cpu_utilization_warm(cpu_util_warm.into());
                        app.set_cpu_utilization_hot(cpu_util_hot.into());
                        app.set_cpu_temperature_cool(cpu_temp_cool.into());
                        app.set_cpu_temperature_warm(cpu_temp_warm.into());
                        app.set_cpu_temperature_hot(cpu_temp_hot.into());
                        app.set_gpu_utilization_cool(gpu_util_cool.into());
                        app.set_gpu_utilization_warm(gpu_util_warm.into());
                        app.set_gpu_utilization_hot(gpu_util_hot.into());
                        app.set_gpu_temperature_cool(gpu_temp_cool.into());
                        app.set_gpu_temperature_warm(gpu_temp_warm.into());
                        app.set_gpu_temperature_hot(gpu_temp_hot.into());
                        app.set_gpu_hotspot_cool(gpu_hotspot_cool.into());
                        app.set_gpu_hotspot_warm(gpu_hotspot_warm.into());
                        app.set_gpu_hotspot_hot(gpu_hotspot_hot.into());
                        app.set_gpu_vram_cool(vram_cool.into());
                        app.set_gpu_vram_warm(vram_warm.into());
                        app.set_gpu_vram_hot(vram_hot.into());
                        app.set_ram_cool(ram_cool.into());
                        app.set_ram_warm(ram_warm.into());
                        app.set_ram_hot(ram_hot.into());
                        match snapshot {
                            Some(m) => {
                                if let Some(cpu) = m.cpu {
                                    app.set_cpu_utilization(format_metric(cpu.utilization).into());
                                    app.set_cpu_utilization_state(to_tile_state(cpu_util_state(
                                        cpu.utilization,
                                    )));
                                    app.set_cpu_temperature(format_metric(cpu.temperature).into());
                                    app.set_cpu_temperature_state(to_tile_state(cpu_temp_state(
                                        cpu.temperature,
                                    )));
                                }
                                if let Some(gpu) = m.gpu {
                                    app.set_gpu_utilization(format_metric(gpu.utilization).into());
                                    app.set_gpu_utilization_state(to_tile_state(gpu_util_state(
                                        gpu.utilization,
                                    )));
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
                                app.set_cpu_utilization_state(TileState::Normal);
                                app.set_cpu_temperature("--".into());
                                app.set_cpu_temperature_state(TileState::Normal);
                                app.set_gpu_utilization("--".into());
                                app.set_gpu_utilization_state(TileState::Normal);
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
    use crate::proto::sentinel::v1::{GpuMetrics, RamMetrics};
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
    fn to_tile_state_maps_all_variants() {
        assert_eq!(to_tile_state(MetricState::Normal), TileState::Normal);
        assert_eq!(to_tile_state(MetricState::Warm), TileState::Warm);
        assert_eq!(to_tile_state(MetricState::Hot), TileState::Hot);
    }

    #[test]
    fn format_uptime_zero_seconds() {
        assert_eq!(format_uptime(0), "0s");
    }

    #[test]
    fn format_fraction_gb_zero_used() {
        assert_eq!(format_fraction_gb(0, 32768), "0.0 / 32.0");
    }

    #[test]
    fn build_colored_all_three_states_present() {
        let state_fn = |v: f32| {
            if v >= 90.0 {
                MetricState::Hot
            } else if v >= 70.0 {
                MetricState::Warm
            } else {
                MetricState::Normal
            }
        };
        let (cool, warm, hot) = build_colored_sparkline_paths(&[10.0, 75.0, 95.0, 10.0], state_fn);
        assert!(!cool.is_empty());
        assert!(!warm.is_empty());
        assert!(!hot.is_empty());
    }

    #[test]
    fn update_display_totals_retains_vram_when_gpu_absent() {
        let mut last_vram = 0u64;
        let mut last_ram = 0u64;
        update_display_totals(
            Some(MetricsSnapshot {
                gpu: Some(GpuMetrics {
                    vram_total_mb: 8192,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            &mut last_vram,
            &mut last_ram,
        );
        assert_eq!(last_vram, 8192);
        update_display_totals(
            Some(MetricsSnapshot {
                ..Default::default()
            }),
            &mut last_vram,
            &mut last_ram,
        );
        assert_eq!(last_vram, 8192);
    }

    #[test]
    fn update_display_totals_retains_ram_when_ram_absent() {
        let mut last_vram = 0u64;
        let mut last_ram = 0u64;
        update_display_totals(
            Some(MetricsSnapshot {
                ram: Some(RamMetrics {
                    total_mb: 32768,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            &mut last_vram,
            &mut last_ram,
        );
        assert_eq!(last_ram, 32768);
        update_display_totals(
            Some(MetricsSnapshot {
                ..Default::default()
            }),
            &mut last_vram,
            &mut last_ram,
        );
        assert_eq!(last_ram, 32768);
    }

    #[test]
    fn update_display_totals_resets_on_none_snapshot() {
        let mut last_vram = 8192u64;
        let mut last_ram = 32768u64;
        update_display_totals(None, &mut last_vram, &mut last_ram);
        assert_eq!(last_vram, 0);
        assert_eq!(last_ram, 0);
    }

    #[test]
    fn build_colored_empty_returns_three_empty_strings() {
        let (cool, warm, hot) = build_colored_sparkline_paths(&[], |_| MetricState::Normal);
        assert_eq!(cool, "");
        assert_eq!(warm, "");
        assert_eq!(hot, "");
    }

    #[test]
    fn build_colored_single_point_returns_three_empty_strings() {
        let (cool, warm, hot) = build_colored_sparkline_paths(&[42.0], |_| MetricState::Normal);
        assert_eq!(cool, "");
        assert_eq!(warm, "");
        assert_eq!(hot, "");
    }

    #[test]
    fn build_colored_all_normal_fills_only_cool_path() {
        let (cool, warm, hot) =
            build_colored_sparkline_paths(&[10.0, 20.0, 30.0], |_| MetricState::Normal);
        assert!(!cool.is_empty(), "cool path should not be empty");
        assert_eq!(warm, "");
        assert_eq!(hot, "");
    }

    #[test]
    fn build_colored_flat_all_normal_uses_midpoint_y() {
        let (cool, _, _) = build_colored_sparkline_paths(&[5.0, 5.0, 5.0], |_| MetricState::Normal);
        assert!(cool.starts_with("M 0.0,50.0"), "got: {cool}");
        assert!(cool.ends_with("L 100.0,50.0"), "got: {cool}");
    }

    #[test]
    fn build_colored_transition_splits_at_boundary_point() {
        let state_fn = |v: f32| {
            if v >= 70.0 {
                MetricState::Warm
            } else {
                MetricState::Normal
            }
        };
        let (cool, warm, hot) = build_colored_sparkline_paths(&[50.0, 80.0, 30.0], state_fn);
        assert_eq!(warm, "M 0.0,60.0 L 50.0,0.0");
        assert_eq!(cool, "M 50.0,0.0 L 100.0,100.0");
        assert_eq!(hot, "");
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
        assert_eq!(app.get_cpu_utilization_state(), TileState::Normal);
        assert_eq!(app.get_cpu_temperature().as_str(), "--");
        assert_eq!(app.get_gpu_utilization().as_str(), "--");
        assert_eq!(app.get_gpu_utilization_state(), TileState::Normal);
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
