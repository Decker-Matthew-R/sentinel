use crate::connection_state::ConnectionState;
use crate::framing::read_frame;
use crate::proto::sentinel::v1::envelope;
use crate::state_store::StateStore;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::time::timeout;

pub struct LinkServer {
    listener: TcpListener,
}

impl LinkServer {
    pub async fn bind(port: u16) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(("0.0.0.0", port)).await?;
        Ok(Self { listener })
    }

    #[allow(dead_code)]
    pub fn local_port(&self) -> u16 {
        self.listener.local_addr().unwrap().port()
    }

    pub async fn run(
        &self,
        tx: watch::Sender<ConnectionState>,
        store: &StateStore,
        heartbeat_timeout: Duration,
    ) {
        loop {
            let (mut stream, _) = match self.listener.accept().await {
                Ok(s) => s,
                Err(_) => continue,
            };

            let envelope = match read_frame(&mut stream).await {
                Ok(e) => e,
                Err(_) => continue,
            };

            let agent_version = match envelope.payload {
                Some(envelope::Payload::Hello(h)) => h.agent_version,
                _ => continue,
            };

            println!("[Connected] agent: {agent_version}");
            tx.send(ConnectionState::Connected {
                agent_version: agent_version.clone(),
                uptime_seconds: 0,
            })
            .ok();

            let mut count = 0u64;
            loop {
                match timeout(heartbeat_timeout, read_frame(&mut stream)).await {
                    Ok(Ok(envelope)) => {
                        count += 1;
                        match envelope.payload {
                            Some(envelope::Payload::MetricsSnapshot(m)) => {
                                tx.send(ConnectionState::Connected {
                                    agent_version: agent_version.clone(),
                                    uptime_seconds: m.uptime_seconds,
                                })
                                .ok();
                                store.update(m);
                            }
                            Some(envelope::Payload::Specs(s)) => {
                                store.update_specs(s);
                            }
                            _ => {}
                        }
                        println!("[Heartbeat] {count}");
                    }
                    _ => {
                        println!("[Stale] agent: {agent_version}");
                        store.clear();
                        tx.send(ConnectionState::Stale {
                            agent_version: agent_version.clone(),
                        })
                        .ok();
                        break;
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn accept_hello(&self) -> Result<String, std::io::Error> {
        let (mut stream, _addr) = self.listener.accept().await?;
        let envelope = read_frame(&mut stream).await?;
        match envelope.payload {
            Some(envelope::Payload::Hello(hello)) => Ok(hello.agent_version),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "expected Hello as first frame",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection_state::ConnectionState;
    use crate::framing::write_frame;
    use crate::proto::sentinel::v1::{
        CpuMetrics, Envelope, Heartbeat, Hello, MetricsSnapshot, Specs, envelope,
    };
    use crate::state_store::StateStore;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;
    use tokio::sync::watch;

    fn hello_envelope(version: &str) -> Envelope {
        Envelope {
            version: 1,
            payload: Some(envelope::Payload::Hello(Hello {
                agent_version: version.into(),
            })),
        }
    }

    #[tokio::test]
    async fn state_becomes_connected_after_hello() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Connected {
                agent_version: "0.1.0".into(),
                uptime_seconds: 0,
            }
        );
    }

    #[tokio::test]
    async fn state_becomes_stale_on_heartbeat_timeout() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_millis(100)).await });

        let mut _stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut _stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected")
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Stale")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Stale {
                agent_version: "0.1.0".into()
            }
        );
    }

    #[tokio::test]
    async fn state_becomes_stale_on_disconnect() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(&mut stream, &hello_envelope("0.1.0"))
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(2), rx.changed())
                .await
                .expect("state did not transition to Connected")
                .unwrap();
        }

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Stale")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Stale {
                agent_version: "0.1.0".into()
            }
        );
    }

    #[tokio::test]
    async fn state_reconnects_to_connected_after_stale() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(&mut stream, &hello_envelope("0.1.0"))
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(2), rx.changed())
                .await
                .expect("state did not transition to Connected")
                .unwrap();
        }
        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Stale")
            .unwrap();
        assert!(matches!(*rx.borrow(), ConnectionState::Stale { .. }));

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.2.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not reconnect to Connected")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Connected {
                agent_version: "0.2.0".into(),
                uptime_seconds: 0,
            }
        );
    }

    #[tokio::test]
    async fn run_skips_non_hello_first_frame_and_accepts_next() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(
                &mut stream,
                &Envelope {
                    version: 1,
                    payload: Some(envelope::Payload::Heartbeat(Heartbeat {})),
                },
            )
            .await
            .unwrap();
        }

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected after bad first frame")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Connected {
                agent_version: "0.1.0".into(),
                uptime_seconds: 0,
            }
        );
    }

    #[tokio::test]
    async fn run_skips_malformed_first_frame_and_accepts_next() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            stream.write_all(&[0x00, 0x00, 0x00, 0x10]).await.unwrap();
            stream.write_all(&[0xFF, 0xFF]).await.unwrap();
        }

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected after malformed first frame")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Connected {
                agent_version: "0.1.0".into(),
                uptime_seconds: 0,
            }
        );
    }

    #[tokio::test]
    async fn store_histories_are_cleared_on_disconnect() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, mut store_rx) = StateStore::new();
        let store = Arc::new(store);

        tokio::spawn({
            let store = Arc::clone(&store);
            async move { server.run(tx, &store, Duration::from_secs(5)).await }
        });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(&mut stream, &hello_envelope("0.1.0"))
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(2), rx.changed())
                .await
                .expect("state did not transition to Connected")
                .unwrap();
            write_frame(
                &mut stream,
                &Envelope {
                    version: 1,
                    payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                        uptime_seconds: 10,
                        cpu: Some(CpuMetrics {
                            utilization: 50.0,
                            temperature: 60.0,
                        }),
                        ..Default::default()
                    })),
                },
            )
            .await
            .unwrap();
            tokio::time::timeout(Duration::from_secs(2), store_rx.changed())
                .await
                .expect("store was not updated")
                .unwrap();
        }

        tokio::time::timeout(Duration::from_secs(2), store_rx.changed())
            .await
            .expect("store was not cleared on disconnect")
            .unwrap();
        assert!(store.histories().cpu_utilization.is_empty());
    }

    #[tokio::test]
    async fn uptime_is_updated_from_metrics_snapshot() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected")
            .unwrap();

        write_frame(
            &mut stream,
            &Envelope {
                version: 1,
                payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                    uptime_seconds: 42,
                    ..Default::default()
                })),
            },
        )
        .await
        .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not update uptime")
            .unwrap();
        assert_eq!(
            *rx.borrow(),
            ConnectionState::Connected {
                agent_version: "0.1.0".into(),
                uptime_seconds: 42,
            }
        );
    }

    #[tokio::test]
    async fn store_is_updated_when_metrics_snapshot_arrives() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, mut store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected")
            .unwrap();

        write_frame(
            &mut stream,
            &Envelope {
                version: 1,
                payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                    uptime_seconds: 99,
                    ..Default::default()
                })),
            },
        )
        .await
        .unwrap();

        tokio::time::timeout(Duration::from_secs(2), store_rx.changed())
            .await
            .expect("store was not updated")
            .unwrap();
        assert_eq!(store_rx.borrow().as_ref().unwrap().uptime_seconds, 99);
    }

    #[tokio::test]
    async fn specs_message_is_stored_in_state_store() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();
        let store = Arc::new(store);
        let mut specs_rx = store.subscribe_specs();

        tokio::spawn({
            let store = Arc::clone(&store);
            async move { server.run(tx, &store, Duration::from_secs(5)).await }
        });

        let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        write_frame(&mut stream, &hello_envelope("0.1.0"))
            .await
            .unwrap();

        tokio::time::timeout(Duration::from_secs(2), rx.changed())
            .await
            .expect("state did not transition to Connected")
            .unwrap();

        write_frame(
            &mut stream,
            &Envelope {
                version: 1,
                payload: Some(envelope::Payload::Specs(Specs {
                    cpu_name: "Test CPU".into(),
                    ..Default::default()
                })),
            },
        )
        .await
        .unwrap();

        tokio::time::timeout(Duration::from_secs(2), specs_rx.changed())
            .await
            .expect("specs were not stored in state store")
            .unwrap();
        assert_eq!(specs_rx.borrow().as_ref().unwrap().cpu_name, "Test CPU");
    }

    #[tokio::test]
    async fn specs_cleared_on_disconnect() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, _store_rx) = StateStore::new();
        let store = Arc::new(store);
        let mut specs_rx = store.subscribe_specs();

        tokio::spawn({
            let store = Arc::clone(&store);
            async move { server.run(tx, &store, Duration::from_secs(5)).await }
        });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(&mut stream, &hello_envelope("0.1.0"))
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(2), rx.changed())
                .await
                .expect("state did not transition to Connected")
                .unwrap();
            write_frame(
                &mut stream,
                &Envelope {
                    version: 1,
                    payload: Some(envelope::Payload::Specs(Specs {
                        cpu_name: "Test CPU".into(),
                        ..Default::default()
                    })),
                },
            )
            .await
            .unwrap();
            tokio::time::timeout(Duration::from_secs(2), specs_rx.changed())
                .await
                .expect("specs were not stored")
                .unwrap();
            assert!(specs_rx.borrow().is_some());
        }

        tokio::time::timeout(Duration::from_secs(2), specs_rx.changed())
            .await
            .expect("specs were not cleared on disconnect")
            .unwrap();
        assert!(specs_rx.borrow().is_none());
    }

    #[tokio::test]
    async fn store_is_cleared_on_disconnect() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();
        let (tx, mut rx) = watch::channel(ConnectionState::Waiting);
        let (store, mut store_rx) = StateStore::new();

        tokio::spawn(async move { server.run(tx, &store, Duration::from_secs(5)).await });

        {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(&mut stream, &hello_envelope("0.1.0"))
                .await
                .unwrap();
            tokio::time::timeout(Duration::from_secs(2), rx.changed())
                .await
                .expect("state did not transition to Connected")
                .unwrap();
            write_frame(
                &mut stream,
                &Envelope {
                    version: 1,
                    payload: Some(envelope::Payload::MetricsSnapshot(MetricsSnapshot {
                        uptime_seconds: 10,
                        ..Default::default()
                    })),
                },
            )
            .await
            .unwrap();
            tokio::time::timeout(Duration::from_secs(2), store_rx.changed())
                .await
                .expect("store was not updated")
                .unwrap();
        }

        tokio::time::timeout(Duration::from_secs(2), store_rx.changed())
            .await
            .expect("store was not cleared on disconnect")
            .unwrap();
        assert!(store_rx.borrow().is_none());
    }
}
