use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use sqlx_core::net::Socket;

/// Adapter that wraps an sqlx-core `Socket` to implement `futures_io::AsyncRead + AsyncWrite`,
/// which is what tiberius requires.
pub(crate) struct SocketAdapter<S: Socket> {
    inner: S,
}

impl<S: Socket> SocketAdapter<S> {
    pub fn new(socket: S) -> Self {
        Self { inner: socket }
    }
}

impl<S: Socket> futures_io::AsyncRead for SocketAdapter<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.inner.try_read(&mut &mut *buf) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    match self.inner.poll_read_ready(cx) {
                        Poll::Ready(Ok(())) => continue,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}

impl<S: Socket> futures_io::AsyncWrite for SocketAdapter<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.inner.try_write(buf) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    match self.inner.poll_write_ready(cx) {
                        Poll::Ready(Ok(())) => continue,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Pending => return Poll::Pending,
                    }
                }
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.inner.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.inner.poll_shutdown(cx)
    }
}

// Implement Unpin since we only access the inner socket through &mut self
impl<S: Socket> Unpin for SocketAdapter<S> {}
