mod proto {
    pub mod sentinel {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/sentinel.v1.rs"));
        }
    }
}

use prost::Message;
use proto::sentinel::v1::{
    envelope, CpuMetrics, Envelope, GpuMetrics, Hello, MetricsSnapshot, RamMetrics, Specs,
};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

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
            })),
        },
    )
    .await?;

    println!("Connected. Streaming metrics...");

    let scenarios = scenarios();
    let mut tick = 0u64;
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let scenario_index = (tick / TICKS_PER_SCENARIO) as usize % scenarios.len();
        let scenario = &scenarios[scenario_index];
        let snapshot = build_snapshot(tick, scenario);

        write_frame(
            &mut stream,
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
}

#[tokio::main]
async fn main() {
    println!("Connecting to panel at 127.0.0.1:9000...");
    run("127.0.0.1:9000").await.expect("connection failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    async fn read_frame_raw(stream: &mut tokio::net::TcpStream) -> Envelope {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await.unwrap();
        let len = u32::from_be_bytes(len_bytes) as usize;
        let mut payload = vec![0u8; len];
        stream.read_exact(&mut payload).await.unwrap();
        Envelope::decode(payload.as_slice()).unwrap()
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
}
