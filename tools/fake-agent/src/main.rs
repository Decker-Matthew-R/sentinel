mod proto {
    pub mod sentinel {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/sentinel.v1.rs"));
        }
    }
}

use prost::Message;
use proto::sentinel::v1::{
    envelope, CommandResult, CpuMetrics, Envelope, GpuMetrics, Hello, MetricsSnapshot, RamMetrics,
    Specs,
};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

async fn write_frame<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    envelope: &Envelope,
) -> Result<(), std::io::Error> {
    let payload = envelope.encode_to_vec();
    let length = (payload.len() as u32).to_be_bytes();
    writer.write_all(&length).await?;
    writer.write_all(&payload).await?;
    Ok(())
}

async fn read_frame<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<Envelope, std::io::Error> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    Envelope::decode(payload.as_slice())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

struct Scenario {
    label: &'static str,
    cpu_utilization: f32,
    cpu_temperature: f32,
    gpu: Option<GpuScenario>,
    ram_used_mb: u64,
    ram_total_mb: u64,
}

struct GpuScenario {
    utilization: f32,
    temperature: f32,
    hotspot_temperature: f32,
    vram_used_mb: u64,
    vram_total_mb: u64,
}

const TICKS_PER_SCENARIO: u64 = 8;

fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            label: "Idle",
            cpu_utilization: 6.0,
            cpu_temperature: 38.0,
            gpu: Some(GpuScenario {
                utilization: 3.0,
                temperature: 32.0,
                hotspot_temperature: 38.0,
                vram_used_mb: 1229,
                vram_total_mb: 16384,
            }),
            ram_used_mb: 8704,
            ram_total_mb: 32768,
        },
        Scenario {
            label: "Gaming — Normal",
            cpu_utilization: 45.0,
            cpu_temperature: 62.0,
            gpu: Some(GpuScenario {
                utilization: 78.0,
                temperature: 72.0,
                hotspot_temperature: 84.0,
                vram_used_mb: 8704,
                vram_total_mb: 16384,
            }),
            ram_used_mb: 14541,
            ram_total_mb: 32768,
        },
        Scenario {
            label: "Gaming — Warm",
            cpu_utilization: 72.0,
            cpu_temperature: 74.0,
            gpu: Some(GpuScenario {
                utilization: 92.0,
                temperature: 82.0,
                hotspot_temperature: 93.0,
                vram_used_mb: 12288,
                vram_total_mb: 16384,
            }),
            ram_used_mb: 22630,
            ram_total_mb: 32768,
        },
        Scenario {
            label: "Gaming — Critical",
            cpu_utilization: 95.0,
            cpu_temperature: 91.0,
            gpu: Some(GpuScenario {
                utilization: 99.0,
                temperature: 93.0,
                hotspot_temperature: 105.0,
                vram_used_mb: 16179,
                vram_total_mb: 16384,
            }),
            ram_used_mb: 29082,
            ram_total_mb: 32768,
        },
        Scenario {
            label: "No GPU",
            cpu_utilization: 25.0,
            cpu_temperature: 48.0,
            gpu: None,
            ram_used_mb: 9421,
            ram_total_mb: 32768,
        },
    ]
}

fn build_snapshot(tick: u64, scenario: &Scenario) -> MetricsSnapshot {
    MetricsSnapshot {
        uptime_seconds: tick,
        cpu: Some(CpuMetrics {
            utilization: scenario.cpu_utilization,
            temperature: scenario.cpu_temperature,
        }),
        gpu: scenario.gpu.as_ref().map(|g| GpuMetrics {
            utilization: g.utilization,
            temperature: g.temperature,
            hotspot_temperature: g.hotspot_temperature,
            vram_used_mb: g.vram_used_mb,
            vram_total_mb: g.vram_total_mb,
        }),
        ram: Some(RamMetrics {
            used_mb: scenario.ram_used_mb,
            total_mb: scenario.ram_total_mb,
        }),
    }
}

pub async fn run(addr: &str) -> Result<(), std::io::Error> {
    let mut stream = TcpStream::connect(addr).await?;

    write_frame(
        &mut stream,
        &Envelope {
            version: 1,
            payload: Some(envelope::Payload::Hello(Hello {
                agent_version: "fake-1.0.0".into(),
            })),
        },
    )
    .await?;

    write_frame(
        &mut stream,
        &Envelope {
            version: 1,
            payload: Some(envelope::Payload::Specs(Specs {
                cpu_name: "Fake CPU i9-9999K".into(),
                gpu_name: "Fake GPU RTX 9090".into(),
                ram_total_mb: 32768,
                os_version: "Fake OS 1.0".into(),
                cpu_core_count: 24,
                cpu_thread_count: 32,
                cpu_boost_clock_mhz: 5800,
                gpu_vram_total_mb: 16384,
                ram_speed_mhz: 6000,
                ram_name: "Fake RAM DDR5-6000".into(),
            })),
        },
    )
    .await?;

    println!("Connected. Streaming metrics...");

    let (mut read_half, mut write_half) = stream.into_split();
    let (inbound_tx, mut inbound_rx) = mpsc::channel::<Envelope>(8);

    tokio::spawn(async move {
        while let Ok(envelope) = read_frame(&mut read_half).await {
            if inbound_tx.send(envelope).await.is_err() {
                break;
            }
        }
    });

    let scenarios = scenarios();
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let scenario_index = (tick / TICKS_PER_SCENARIO) as usize % scenarios.len();
                let scenario = &scenarios[scenario_index];
                let snapshot = build_snapshot(tick, scenario);

                write_frame(
                    &mut write_half,
                    &Envelope {
                        version: 1,
                        payload: Some(envelope::Payload::MetricsSnapshot(snapshot)),
                    },
                )
                .await?;

                let tick_in_scenario = tick % TICKS_PER_SCENARIO + 1;
                println!(
                    "[{}] tick {tick_in_scenario}/{TICKS_PER_SCENARIO} — uptime {}s",
                    scenario.label, tick
                );

                tick += 1;
            }
            maybe_envelope = inbound_rx.recv() => {
                match maybe_envelope {
                    None => break,
                    Some(envelope) => {
                        if let Some(envelope::Payload::Command(cmd)) = envelope.payload {
                            println!("[Command] action_id={}", cmd.action_id);
                            write_frame(
                                &mut write_half,
                                &Envelope {
                                    version: 1,
                                    payload: Some(envelope::Payload::CommandResult(
                                        CommandResult {
                                            request_id: cmd.request_id,
                                            success: true,
                                            error_message: String::new(),
                                        },
                                    )),
                                },
                            )
                            .await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Connecting to panel at 127.0.0.1:9000...");
    run("127.0.0.1:9000").await.expect("connection failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto::sentinel::v1::Command;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn read_frame_raw(stream: &mut tokio::net::TcpStream) -> Envelope {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await.unwrap();
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload).await.unwrap();
        Envelope::decode(payload.as_slice()).unwrap()
    }

    async fn write_frame_raw(stream: &mut tokio::net::TcpStream, envelope: &Envelope) {
        let payload = envelope.encode_to_vec();
        let length = (payload.len() as u32).to_be_bytes();
        stream.write_all(&length).await.unwrap();
        stream.write_all(&payload).await.unwrap();
    }

    #[tokio::test]
    async fn fake_agent_sends_hello_on_connect() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move { run(&format!("127.0.0.1:{port}")).await.ok() });

        let (mut stream, _) = listener.accept().await.unwrap();

        let envelope = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Hello not received within timeout");

        assert!(matches!(
            envelope.payload,
            Some(envelope::Payload::Hello(_))
        ));
    }

    #[tokio::test]
    async fn fake_agent_sends_specs_after_hello() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move { run(&format!("127.0.0.1:{port}")).await.ok() });

        let (mut stream, _) = listener.accept().await.unwrap();

        let first = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Hello not received");
        assert!(matches!(first.payload, Some(envelope::Payload::Hello(_))));

        let second = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Specs not received");
        assert!(matches!(second.payload, Some(envelope::Payload::Specs(_))));
    }

    #[tokio::test]
    async fn fake_agent_sends_metrics_snapshot_after_hello() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move { run(&format!("127.0.0.1:{port}")).await.ok() });

        let (mut stream, _) = listener.accept().await.unwrap();

        // Hello → Specs → MetricsSnapshot
        let first = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Hello not received");
        assert!(matches!(first.payload, Some(envelope::Payload::Hello(_))));

        let second = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Specs not received");
        assert!(matches!(second.payload, Some(envelope::Payload::Specs(_))));

        let third = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("MetricsSnapshot not received");
        assert!(matches!(
            third.payload,
            Some(envelope::Payload::MetricsSnapshot(_))
        ));
    }

    #[tokio::test]
    async fn fake_agent_specs_fields_are_populated() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move { run(&format!("127.0.0.1:{port}")).await.ok() });

        let (mut stream, _) = listener.accept().await.unwrap();

        let _hello = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Hello not received");

        let specs_envelope =
            tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
                .await
                .expect("Specs not received");

        match specs_envelope.payload {
            Some(envelope::Payload::Specs(s)) => {
                assert!(!s.cpu_name.is_empty());
                assert!(!s.gpu_name.is_empty());
                assert!(s.ram_total_mb > 0);
                assert!(!s.os_version.is_empty());
                assert!(s.cpu_core_count > 0);
                assert!(s.cpu_thread_count > 0);
                assert!(s.cpu_boost_clock_mhz > 0);
                assert!(s.gpu_vram_total_mb > 0);
                assert!(s.ram_speed_mhz > 0);
                assert!(!s.ram_name.is_empty());
            }
            _ => panic!("expected Specs payload"),
        }
    }

    #[tokio::test]
    async fn build_snapshot_includes_cpu_and_ram() {
        let s = &scenarios()[0];
        let snapshot = build_snapshot(0, s);
        assert!(snapshot.cpu.is_some());
        assert!(snapshot.ram.is_some());
    }

    #[tokio::test]
    async fn no_gpu_scenario_omits_gpu() {
        let no_gpu = scenarios().into_iter().find(|s| s.gpu.is_none()).unwrap();
        let snapshot = build_snapshot(0, &no_gpu);
        assert!(snapshot.gpu.is_none());
    }

    #[tokio::test]
    async fn fake_agent_replies_to_command_with_command_result() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move { run(&format!("127.0.0.1:{port}")).await.ok() });

        let (mut stream, _) = listener.accept().await.unwrap();

        // Skip Hello and Specs
        let _hello = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Hello not received");
        let _specs = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Specs not received");

        // Send a Command
        let request_id = "test-req-42".to_string();
        write_frame_raw(
            &mut stream,
            &Envelope {
                version: 1,
                payload: Some(envelope::Payload::Command(Command {
                    request_id: request_id.clone(),
                    action_id: "launch_steam".into(),
                })),
            },
        )
        .await;

        // Read frames until we receive a CommandResult — MetricsSnapshots may arrive first
        for _ in 0..5 {
            let envelope =
                tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
                    .await
                    .expect("frame not received");

            if let Some(envelope::Payload::CommandResult(result)) = envelope.payload {
                assert_eq!(result.request_id, request_id);
                assert!(result.success);
                assert!(result.error_message.is_empty());
                return;
            }
        }

        panic!("CommandResult not received within 5 frames");
    }
}
