#![allow(dead_code)]

use crate::proto::sentinel::v1::Envelope;
use prost::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn write_frame<W: tokio::io::AsyncWrite + Unpin>(writer: &mut W, envelope: &Envelope) {
    let payload = envelope.encode_to_vec();
    let length = (payload.len() as u32).to_be_bytes();
    writer.write_all(&length).await.unwrap();
    writer.write_all(&payload).await.unwrap();
}

pub async fn read_frame<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> Envelope {
    let mut length_bytes = [0u8; 4];
    reader.read_exact(&mut length_bytes).await.unwrap();

    let length = u32::from_be_bytes(length_bytes) as usize;

    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload).await.unwrap();

    Envelope::decode(payload.as_slice()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::sentinel::v1::{Heartbeat, MetricsSnapshot, envelope};

    #[tokio::test]
    async fn write_and_read_round_trip_a_single_frame() {
        let original = Envelope {
            version: 1,
            payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                uptime_seconds: 9999,
            })),
        };

        let (mut writer, mut reader) = tokio::io::duplex(1024);
        write_frame(&mut writer, &original).await;
        drop(writer);

        let decoded = read_frame(&mut reader).await;

        assert_eq!(decoded.version, 1);
        match decoded.payload {
            Some(envelope::Payload::MetricsSnapshot(m)) => assert_eq!(m.uptime_seconds, 9999),
            _ => panic!("unexpected payload variant"),
        }
    }

    #[tokio::test]
    async fn read_frame_handles_multiple_consecutive_frames() {
        let first = Envelope {
            version: 1,
            payload: Some(envelope::Payload::Heartbeat(Heartbeat {})),
        };
        let second = Envelope {
            version: 1,
            payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                uptime_seconds: 42,
            })),
        };

        let (mut writer, mut reader) = tokio::io::duplex(1024);
        write_frame(&mut writer, &first).await;
        write_frame(&mut writer, &second).await;
        drop(writer);

        let decoded_first = read_frame(&mut reader).await;
        let decoded_second = read_frame(&mut reader).await;

        assert!(matches!(
            decoded_first.payload,
            Some(envelope::Payload::Heartbeat(_))
        ));
        match decoded_second.payload {
            Some(envelope::Payload::MetricsSnapshot(m)) => assert_eq!(m.uptime_seconds, 42),
            _ => panic!("unexpected payload variant"),
        }
    }
}
