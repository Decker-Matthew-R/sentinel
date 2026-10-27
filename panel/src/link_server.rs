use crate::framing::read_frame;
use crate::proto::sentinel::v1::envelope;
use tokio::net::TcpListener;

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
    use crate::framing::write_frame;
    use crate::proto::sentinel::v1::{Envelope, Heartbeat, Hello, envelope};
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn accept_hello_returns_agent_version() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();

        let client_task = tokio::spawn(async move {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            write_frame(
                &mut stream,
                &Envelope {
                    version: 1,
                    payload: Some(envelope::Payload::Hello(Hello {
                        agent_version: "0.1.0".into(),
                    })),
                },
            )
            .await
            .unwrap();
        });

        let result = server.accept_hello().await.unwrap();
        assert_eq!(result, "0.1.0");
        client_task.await.unwrap();
    }

    #[tokio::test]
    async fn non_hello_first_frame_is_rejected() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();

        tokio::spawn(async move {
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
        });

        let result = server.accept_hello().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    }

    #[tokio::test]
    async fn malformed_frame_is_rejected() {
        let server = LinkServer::bind(0).await.unwrap();
        let port = server.local_port();

        tokio::spawn(async move {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            use tokio::io::AsyncWriteExt;
            stream.write_all(&[0x00, 0x00, 0x00, 0x10]).await.unwrap();
            stream.write_all(&[0xFF, 0xFF]).await.unwrap();
        });

        let result = server.accept_hello().await;
        assert!(result.is_err());
    }
}
