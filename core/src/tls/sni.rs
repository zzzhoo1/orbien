use anyhow::{anyhow, bail, Result};
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tls_parser::{
    parse_tls_extensions, parse_tls_plaintext, SNIType, TlsExtension, TlsMessage,
    TlsMessageHandshake,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};

const MAX_RECORD: usize = 5 + 16 * 1024;

pub async fn peek_client_hello_sni<R: AsyncRead + Unpin>(
    reader: &mut R,
) -> Result<(String, Vec<u8>)> {
    let mut buf = Vec::with_capacity(1024);
    let mut tmp = [0u8; 2048];

    while buf.len() < 5 {
        let n = reader.read(&mut tmp).await?;
        if n == 0 {
            bail!("connection closed before TLS ClientHello");
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > MAX_RECORD {
            bail!("TLS preamble too large");
        }
    }

    if buf[0] != 0x16 {
        bail!("not a TLS handshake record (type={:#x})", buf[0]);
    }

    let record_len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
    let need = 5 + record_len;
    if need > MAX_RECORD {
        bail!("TLS record too large: {record_len}");
    }

    while buf.len() < need {
        let n = reader.read(&mut tmp).await?;
        if n == 0 {
            bail!("connection closed mid ClientHello");
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > MAX_RECORD {
            bail!("TLS preamble too large");
        }
    }

    let (record, rest) = buf.split_at(need);
    let sni = extract_sni_from_handshake_record(record)
        .ok_or_else(|| anyhow!("TLS ClientHello missing SNI"))?;
    let mut prefix = record.to_vec();
    prefix.extend_from_slice(rest);
    Ok((sni, prefix))
}

fn extract_sni_from_handshake_record(record: &[u8]) -> Option<String> {
    let (_, plain) = parse_tls_plaintext(record).ok()?;
    for msg in plain.msg {
        let TlsMessage::Handshake(TlsMessageHandshake::ClientHello(hello)) = msg else {
            continue;
        };
        let exts = hello.ext?;
        let (_, parsed) = parse_tls_extensions(exts).ok()?;
        for ext in parsed {
            let TlsExtension::SNI(list) = ext else {
                continue;
            };
            for (name_type, data) in list {
                if name_type == SNIType::HostName {
                    let host = std::str::from_utf8(data).ok()?;
                    return Some(host.to_ascii_lowercase());
                }
            }
        }
    }
    None
}

pub struct PrefixedStream<S> {
    prefix: Vec<u8>,
    pos: usize,
    inner: S,
}

impl<S> PrefixedStream<S> {
    pub fn new(prefix: Vec<u8>, inner: S) -> Self {
        Self {
            prefix,
            pos: 0,
            inner,
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for PrefixedStream<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if this.pos < this.prefix.len() {
            let remain = &this.prefix[this.pos..];
            let n = remain.len().min(buf.remaining());
            buf.put_slice(&remain[..n]);
            this.pos += n;
            return Poll::Ready(Ok(()));
        }
        Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for PrefixedStream<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
