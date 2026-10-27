pub mod sentinel {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/sentinel.v1.rs"));
    }
}

#[cfg(test)]
mod tests {
    use super::sentinel::v1::{Envelope, MetricsSnapshot, envelope};
    use prost::Message;

    #[test]
    fn metrics_snapshot_survives_round_trip() {
        let original = Envelope {
            version: 1,
            payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                uptime_seconds: 3600,
                ..Default::default()
            })),
        };

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();
        let decoded = Envelope::decode(buf.as_slice()).unwrap();

        assert_eq!(decoded.version, 1);
        match decoded.payload {
            Some(envelope::Payload::MetricsSnapshot(m)) => assert_eq!(m.uptime_seconds, 3600),
            _ => panic!("unexpected payload variant"),
        }
    }

    #[test]
    fn heartbeat_survives_round_trip() {
        use super::sentinel::v1::Heartbeat;

        let original = Envelope {
            version: 1,
            payload: Some(envelope::Payload::Heartbeat(Heartbeat {})),
        };

        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();
        let decoded = Envelope::decode(buf.as_slice()).unwrap();

        assert_eq!(decoded.version, 1);
        assert!(matches!(
            decoded.payload,
            Some(envelope::Payload::Heartbeat(_))
        ));
    }
}
