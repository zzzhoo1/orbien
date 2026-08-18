use super::types::*;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
}

#[derive(Debug, Error)]
pub enum MessageWriteError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

const MAX_MSG_SIZE: u32 = 256 * 1024;

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
