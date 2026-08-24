use super::types::*;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::Duration;

#[derive(Debug, Error)]
pub enum MessageReadError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown message type: {0}")]
    UnknownType(u8),
    #[error("message body too large: {0} bytes")]
    TooLarge(u32),
    #[error("read timed out after {0:?}")]
    Timeout(Duration),
}

#[derive(Debug, Error)]
pub enum MessageWriteError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

const MAX_MSG_SIZE: u32 = 256 * 1024;

/// Default timeout applied when using [`read_msg_timeout`].
#[allow(dead_code)]
pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);

pub async fn write_msg<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &Message,
) -> Result<(), MessageWriteError> {
    let type_byte = msg.type_byte();
    let body = match msg {
        Message::Login(m) => serde_json::to_vec(m)?,
        Message::LoginResp(m) => serde_json::to_vec(m)?,
        Message::NewProxy(m) => serde_json::to_vec(m)?,
        Message::NewProxyResp(m) => serde_json::to_vec(m)?,
        Message::CloseProxy(m) => serde_json::to_vec(m)?,
        Message::ReqWorkConn(m) => serde_json::to_vec(m)?,
        Message::NewWorkConn(m) => serde_json::to_vec(m)?,
        Message::StartWorkConn(m) => serde_json::to_vec(m)?,
        Message::Ping(m) => serde_json::to_vec(m)?,
        Message::Pong(m) => serde_json::to_vec(m)?,
        Message::UdpPacket(m) => serde_json::to_vec(m)?,
        Message::KickOut(m) => serde_json::to_vec(m)?,
    };

    writer.write_u8(type_byte).await?;
    writer.write_u32_le(body.len() as u32).await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_msg<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Message, MessageReadError> {
    let type_byte = reader.read_u8().await?;
    let len = reader.read_u32_le().await?;
    if len > MAX_MSG_SIZE {
        return Err(MessageReadError::TooLarge(len));
    }
    let mut buf = vec![0u8; len as usize];
    reader.read_exact(&mut buf).await?;

    let msg = match type_byte {
        TYPE_LOGIN => Message::Login(serde_json::from_slice(&buf)?),
        TYPE_LOGIN_RESP => Message::LoginResp(serde_json::from_slice(&buf)?),
        TYPE_NEW_PROXY => Message::NewProxy(serde_json::from_slice(&buf)?),
        TYPE_NEW_PROXY_RESP => Message::NewProxyResp(serde_json::from_slice(&buf)?),
        TYPE_CLOSE_PROXY => Message::CloseProxy(serde_json::from_slice(&buf)?),
        TYPE_REQ_WORK_CONN => Message::ReqWorkConn(serde_json::from_slice(&buf)?),
        TYPE_NEW_WORK_CONN => Message::NewWorkConn(serde_json::from_slice(&buf)?),
        TYPE_START_WORK_CONN => Message::StartWorkConn(serde_json::from_slice(&buf)?),
        TYPE_PING => Message::Ping(serde_json::from_slice(&buf)?),
        TYPE_PONG => Message::Pong(serde_json::from_slice(&buf)?),
        TYPE_UDP_PACKET => Message::UdpPacket(serde_json::from_slice(&buf)?),
        TYPE_KICK_OUT => Message::KickOut(serde_json::from_slice(&buf)?),
        other => return Err(MessageReadError::UnknownType(other)),
    };
    Ok(msg)
}

/// Like [`read_msg`] but fails with [`MessageReadError::Timeout`] if the
/// full message is not received within `timeout`.
#[allow(dead_code)]
pub async fn read_msg_timeout<R: AsyncRead + Unpin>(
    reader: &mut R,
    timeout: Duration,
) -> Result<Message, MessageReadError> {
    tokio::time::timeout(timeout, read_msg(reader))
        .await
        .unwrap_or(Err(MessageReadError::Timeout(timeout)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    async fn roundtrip(msg: Message) {
        let (mut a, mut b) = duplex(4096);
        write_msg(&mut a, &msg).await.unwrap();
        let got = read_msg(&mut b).await.unwrap();
        assert_eq!(got.type_byte(), msg.type_byte());
        // Compare serialized bodies to ensure identical payloads.
        let (_, body) = split(&msg);
        let (_, got_body) = split(&got);
        assert_eq!(body, got_body, "message body mismatch for type {}", msg.type_byte() as char);
    }

    fn split(msg: &Message) -> (u8, Vec<u8>) {
        let body = match msg {
            Message::Login(m) => serde_json::to_vec(m).unwrap(),
            Message::LoginResp(m) => serde_json::to_vec(m).unwrap(),
            Message::NewProxy(m) => serde_json::to_vec(m).unwrap(),
            Message::NewProxyResp(m) => serde_json::to_vec(m).unwrap(),
            Message::CloseProxy(m) => serde_json::to_vec(m).unwrap(),
            Message::ReqWorkConn(m) => serde_json::to_vec(m).unwrap(),
            Message::NewWorkConn(m) => serde_json::to_vec(m).unwrap(),
            Message::StartWorkConn(m) => serde_json::to_vec(m).unwrap(),
            Message::Ping(m) => serde_json::to_vec(m).unwrap(),
            Message::Pong(m) => serde_json::to_vec(m).unwrap(),
            Message::UdpPacket(m) => serde_json::to_vec(m).unwrap(),
            Message::KickOut(m) => serde_json::to_vec(m).unwrap(),
        };
        (msg.type_byte(), body)
    }

    #[tokio::test]
    async fn login_roundtrip() {
        roundtrip(Message::Login(Login {
            version: "0.1.0".into(),
            hostname: "test-host".into(),
            os: "linux".into(),
            arch: "x86_64".into(),
            user: "alice".into(),
            privilege_key: "abc123".into(),
            timestamp: 1_700_000_000,
            run_id: "run-1".into(),
            pool_count: 4,
        }))
        .await;
    }

    #[tokio::test]
    async fn new_proxy_roundtrip() {
        roundtrip(Message::NewProxy(Box::new(NewProxy {
            proxy_name: "mysql".into(),
            proxy_type: "tcp".into(),
            remote_port: 6050,
            local_ip: "127.0.0.1".into(),
            local_port: 3306,
            custom_domains: vec!["a.example.com".into()],
            subdomain: "db".into(),
            locations: vec![],
            http_user: "".into(),
            http_pwd: "".into(),
            host_header_rewrite: "".into(),
            headers: Default::default(),
            response_headers: Default::default(),
            route_by_http_user: "".into(),
            bandwidth_limit: "10MB".into(),
            bandwidth_limit_mode: "client".into(),
            max_connections: 100,
        })))
        .await;
    }

    #[tokio::test]
    async fn udp_packet_roundtrip() {
        let remote: std::net::SocketAddr = "1.2.3.4:5678".parse().unwrap();
        roundtrip(Message::UdpPacket(UdpPacket::new(
            vec![1, 2, 3, 4, 5],
            Some(remote),
        )))
        .await;
    }

    #[tokio::test]
    async fn ping_pong_roundtrip() {
        roundtrip(Message::Ping(Ping {
            privilege_key: "key".into(),
            timestamp: 123,
        }))
        .await;
        roundtrip(Message::Pong(Pong {
            error: "".into(),
        }))
        .await;
    }

    #[tokio::test]
    async fn kick_out_roundtrip() {
        roundtrip(Message::KickOut(KickOut {
            reason: "duplicate login".into(),
        }))
        .await;
    }

    #[tokio::test]
    async fn rejects_unknown_type() {
        let (mut a, mut b) = duplex(64);
        use tokio::io::AsyncWriteExt;
        a.write_u8(0x7f).await.unwrap(); // unknown type byte
        a.write_u32_le(0).await.unwrap();
        a.flush().await.unwrap();
        let err = read_msg(&mut b).await.unwrap_err();
        assert!(matches!(err, MessageReadError::UnknownType(0x7f)));
    }

    #[tokio::test]
    async fn rejects_oversized_message() {
        let (mut a, mut b) = duplex(64);
        use tokio::io::AsyncWriteExt;
        a.write_u8(TYPE_PING).await.unwrap();
        a.write_u32_le(MAX_MSG_SIZE + 1).await.unwrap();
        a.flush().await.unwrap();
        let err = read_msg(&mut b).await.unwrap_err();
        assert!(matches!(err, MessageReadError::TooLarge(_)));
    }
}
