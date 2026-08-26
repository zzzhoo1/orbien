use super::counting::{ByteCounter, CountingStream};
use tokio::io::{copy_bidirectional_with_sizes, AsyncRead, AsyncWrite};

// Increased from 256KB to 512KB for better throughput on high-bandwidth links.
// Each active connection uses 2 × JOIN_BUF bytes; scale down if memory is tight.
const JOIN_BUF: usize = 512 * 1024;

/// Bidirectional copy that returns `(a_to_b, b_to_a)` on success, or
/// propagates the first I/O error encountered.
pub async fn join<A, B>(a: A, b: B) -> std::io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let (a_to_b, b_to_a, err) = join_counted(a, b).await;
    if let Some(e) = err {
        return Err(e);
    }
    Ok((a_to_b, b_to_a))
}

/// Like [`join`] but never fails: returns byte counts regardless of any error.
/// Use this when the caller genuinely does not care about mid-stream failures
/// (e.g. UDP splice where partial delivery is expected).
#[allow(dead_code)]
pub async fn join_discard_err<A, B>(a: A, b: B) -> (u64, u64)
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let (sent, recv, _err) = join_counted(a, b).await;
    (sent, recv)
}

/// Full result: byte counters **and** the error (if any).
pub async fn join_counted<A, B>(a: A, b: B) -> (u64, u64, Option<std::io::Error>)
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let ca = ByteCounter::new();
    let cb = ByteCounter::new();
    let mut a = CountingStream::new(a, ca.clone());
    let mut b = CountingStream::new(b, cb.clone());

    let err = copy_bidirectional_with_sizes(&mut a, &mut b, JOIN_BUF, JOIN_BUF)
        .await
        .err();

    let a_to_b = ca.read();
    let b_to_a = cb.read();
    (a_to_b, b_to_a, err)
}
