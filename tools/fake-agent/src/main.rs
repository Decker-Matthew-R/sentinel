mod proto {
    pub mod sentinel {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/sentinel.v1.rs"));
        }
    }
}

use prost::Message;
use proto::sentinel::v1::{envelope, Envelope, Hello, MetricsSnapshot};
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
    println!("Connected. Sending heartbeats...");

    let mut count = 0u64;
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        count += 1;
        write_frame(
            &mut stream,
            &Envelope {
                version: 1,
                payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                    uptime_seconds: count,
                })),
            },
        )
        .await?;
        println!("♥ {count}s");
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
    async fn fake_agent_sends_metrics_snapshot_after_hello() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        tokio::spawn(async move { run(&format!("127.0.0.1:{port}")).await.ok() });

        let (mut stream, _) = listener.accept().await.unwrap();

        let first = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("Hello not received within timeout");
        assert!(matches!(first.payload, Some(envelope::Payload::Hello(_))));

        let second = tokio::time::timeout(Duration::from_secs(2), read_frame_raw(&mut stream))
            .await
            .expect("MetricsSnapshot not received within timeout");
        assert!(matches!(
            second.payload,
            Some(envelope::Payload::MetricsSnapshot(_))
        ));
    }
}
