use super::{ConnectionInfo, Plugin, PluginContext};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use httparse::Status;
use orbien_core::config::PluginConfig;
use orbien_core::tls::load_or_generate_https_server_config;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

pub struct TlsTermPlugin {
    local_addr: String,
    host_header_rewrite: String,
    request_headers: Vec<(String, String)>,
    acceptor: TlsAcceptor,
}

impl TlsTermPlugin {
    pub fn new(ctx: PluginContext, cfg: &PluginConfig) -> Result<Self> {
        let local_addr = cfg.service.trim().to_string();
        if local_addr.is_empty() {
            bail!("tls-term requires plugin.service (e.g. \"127.0.0.1:80\")");
        }

        let cn = if ctx.cert_common_name.is_empty() {
            "localhost".to_string()
        } else {
            ctx.cert_common_name.clone()
        };
        let tls_cfg = load_or_generate_https_server_config(&cfg.cert_file, &cfg.key_file, &cn)?;
        let acceptor = TlsAcceptor::from(tls_cfg);

        let request_headers: Vec<(String, String)> = cfg
            .request_headers
            .set
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        tracing::info!(
            tunnel = %ctx.name,
            %local_addr,
            rewrite = %cfg.host_header_rewrite,
            "plugin tls-term ready (TLS terminates on agent)"
        );

        Ok(Self {
            local_addr,
            host_header_rewrite: cfg.host_header_rewrite.clone(),
            request_headers,
            acceptor,
        })
    }
}

#[async_trait]
impl Plugin for TlsTermPlugin {
    fn name(&self) -> &str {
        "tls-term"
    }

    async fn handle(&self, conn: ConnectionInfo) -> Result<()> {
        let tls = self
            .acceptor
            .accept(conn.stream)
            .await
            .map_err(|e| anyhow!("tls-term TLS accept failed: {e}"))?;

        let mut local = TcpStream::connect(&self.local_addr)
            .await
            .map_err(|e| anyhow!("tls-term dial {}: {e}", self.local_addr))?;
        orbien_core::net::enable_nodelay(&local);

        let (mut tls_r, mut tls_w) = tokio::io::split(tls);
        let mut head = read_http_request_head(&mut tls_r).await?;
        apply_request_rewrites(&mut head, &self.host_header_rewrite, &self.request_headers)?;

        orbien_core::net::apply_x_forwarded_for(&mut head, &conn.src_addr, "https")?;
        local.write_all(&head).await?;

        tracing::debug!(
            local = %self.local_addr,
            src = %format!("{}:{}", conn.src_addr, conn.src_port),
            "tls-term joining decrypted <-> local HTTP"
        );

        let (mut local_r, mut local_w) = tokio::io::split(local);
        let _ = tokio::try_join!(
            async move { tokio::io::copy(&mut local_r, &mut tls_w).await },
            async move { tokio::io::copy(&mut tls_r, &mut local_w).await },
        );
        Ok(())
    }
}

async fn read_http_request_head<R: AsyncReadExt + Unpin>(stream: &mut R) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 2048];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            bail!("client closed before http headers completed");
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > 64 * 1024 {
            bail!("http headers too large");
        }
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(&buf)? {
            Status::Complete(_) => return Ok(buf),
            Status::Partial => continue,
        }
    }
}

fn apply_request_rewrites(
    buf: &mut Vec<u8>,
    host_rewrite: &str,
    extra_headers: &[(String, String)],
) -> Result<()> {
    let text = String::from_utf8_lossy(buf);
    let mut lines: Vec<String> = text.split_inclusive('\n').map(|s| s.to_string()).collect();
    if lines.is_empty() {
        bail!("empty http request");
    }

    let ending = if lines.iter().any(|l| l.ends_with("\r\n")) {
        "\r\n"
    } else {
        "\n"
    };

    if !host_rewrite.is_empty() {
        let mut replaced = false;
        for line in lines.iter_mut().skip(1) {
            let trimmed = line.trim_start_matches([' ', '\t']);
            if trimmed.len() >= 5 && trimmed.as_bytes()[..5].eq_ignore_ascii_case(b"host:") {
                *line = format!("Host: {host_rewrite}{ending}");
                replaced = true;
                break;
            }
        }
        if !replaced {
            lines.insert(1, format!("Host: {host_rewrite}{ending}"));
        }
    }

    for (k, v) in extra_headers {
        let blank_idx = lines
            .iter()
            .position(|l| l == "\r\n" || l == "\n")
            .unwrap_or(lines.len());
        lines.insert(blank_idx, format!("{k}: {v}{ending}"));
    }

    *buf = lines.join("").into_bytes();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> Vec<u8> {
        b"GET / HTTP/1.1\r\nHost: original.example.com\r\n\r\n".to_vec()
    }

    #[test]
    fn rewrites_host_header() {
        let mut buf = req();
        apply_request_rewrites(&mut buf, "new.example.com", &[]).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Host: new.example.com\r\n"));
        assert!(!text.contains("original.example.com"));
    }

    #[test]
    fn inserts_host_when_missing() {
        let mut buf = b"GET / HTTP/1.1\r\n\r\n".to_vec();
        apply_request_rewrites(&mut buf, "new.example.com", &[]).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Host: new.example.com\r\n"));
    }

    #[test]
    fn adds_extra_headers() {
        let mut buf = req();
        apply_request_rewrites(&mut buf, "", &[("X-Custom".into(), "val".into())]).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("X-Custom: val\r\n"));
    }

    #[test]
    fn no_rewrite_no_extra_is_noop() {
        let mut buf = req();
        apply_request_rewrites(&mut buf, "", &[]).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "GET / HTTP/1.1\r\nHost: original.example.com\r\n\r\n");
    }

    #[test]
    fn empty_request_errors() {
        let mut buf: Vec<u8> = vec![];
        assert!(apply_request_rewrites(&mut buf, "x", &[]).is_err());
    }
}
