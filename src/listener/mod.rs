use std::io::{self, Error as IoError, ErrorKind};
use std::net::{IpAddr, SocketAddr as StdSocketAddr, ToSocketAddrs};
use std::pin::Pin;
use std::task::{Context, Poll};

use hyper::server::accept::Accept;
use hyper::server::conn::AddrIncoming;
use hyper::server::conn::AddrStream;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use crate::addr::SocketAddr;
use crate::transport::Transport;

pub trait Listener: Accept {
    fn join<T>(self, other: T) -> JoinedListener<Self, T>
    where
        Self: Sized,
    {
        JoinedListener::new(self, other)
    }
}

pub enum JoinedStream<A, B> {
    A(A),
    B(B),
}

impl<A, B> AsyncRead for JoinedStream<A, B>
where
    A: AsyncRead + Send + Unpin + 'static,
    B: AsyncRead + Send + Unpin + 'static,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match &mut self.get_mut() {
            JoinedStream::A(a) => Pin::new(a).poll_read(cx, buf),
            JoinedStream::B(b) => Pin::new(b).poll_read(cx, buf),
        }
    }
}
impl<A, B> AsyncWrite for JoinedStream<A, B>
where
    A: AsyncWrite + Send + Unpin + 'static,
    B: AsyncWrite + Send + Unpin + 'static,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match &mut self.get_mut() {
            JoinedStream::A(a) => Pin::new(a).poll_write(cx, buf),
            JoinedStream::B(b) => Pin::new(b).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match &mut self.get_mut() {
            JoinedStream::A(a) => Pin::new(a).poll_flush(cx),
            JoinedStream::B(b) => Pin::new(b).poll_flush(cx),
        }
    }
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match &mut self.get_mut() {
            JoinedStream::A(a) => Pin::new(a).poll_shutdown(cx),
            JoinedStream::B(b) => Pin::new(b).poll_shutdown(cx),
        }
    }
}

impl<A, B> Transport for JoinedStream<A, B>
where
    A: Transport + Send + Unpin + 'static,
    B: Transport + Send + Unpin + 'static,
{
    fn remote_addr(&self) -> Option<crate::addr::SocketAddr> {
        match self {
            JoinedStream::A(stream) => stream.remote_addr(),
            JoinedStream::B(stream) => stream.remote_addr(),
        }
    }
}

pub struct JoinedListener<A, B> {
    a: A,
    b: B,
}

impl<A, B> JoinedListener<A, B> {
    pub(crate) fn new(a: A, b: B) -> Self {
        JoinedListener { a, b }
    }
}

impl<A, B> Listener for JoinedListener<A, B>
where
    A: Accept + Send + Unpin + 'static,
    B: Accept + Send + Unpin + 'static,
    A::Conn: Transport,
    B::Conn: Transport,
{
}
impl<A, B> Accept for JoinedListener<A, B>
where
    A: Accept + Send + Unpin + 'static,
    B: Accept + Send + Unpin + 'static,
    A::Conn: Transport,
    B::Conn: Transport,
{
    type Conn = JoinedStream<A::Conn, B::Conn>;
    type Error = IoError;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let pin = self.get_mut();
        if fastrand::bool() {
            match Pin::new(&mut pin.a).poll_accept(cx) {
                Poll::Ready(Some(result)) => Poll::Ready(Some(
                    result
                        .map(JoinedStream::A)
                        .map_err(|_| IoError::from(ErrorKind::Other)),
                )),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => match Pin::new(&mut pin.b).poll_accept(cx) {
                    Poll::Ready(Some(result)) => Poll::Ready(Some(
                        result
                            .map(JoinedStream::B)
                            .map_err(|_| IoError::from(ErrorKind::Other)),
                    )),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                },
            }
        } else {
            match Pin::new(&mut pin.b).poll_accept(cx) {
                Poll::Ready(Some(result)) => Poll::Ready(Some(
                    result
                        .map(JoinedStream::B)
                        .map_err(|_| IoError::from(ErrorKind::Other)),
                )),
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Pending => match Pin::new(&mut pin.a).poll_accept(cx) {
                    Poll::Ready(Some(result)) => Poll::Ready(Some(
                        result
                            .map(JoinedStream::A)
                            .map_err(|_| IoError::from(ErrorKind::Other)),
                    )),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                },
            }
        }
    }
}

pub struct TcpListener {
    incoming: AddrIncoming,
}

impl TcpListener {
    pub fn incoming(&self) -> &AddrIncoming {
        &self.incoming
    }
    pub fn local_addr(&self) -> std::net::SocketAddr {
        self.incoming.local_addr()
    }
    pub fn bind(incoming: impl IntoAddrIncoming) -> Self {
        Self::try_bind(incoming).unwrap()
    }
    pub fn try_bind(incoming: impl IntoAddrIncoming) -> Result<Self, hyper::Error> {
        Ok(TcpListener {
            incoming: incoming.into_incoming(),
        })
    }
}

impl Listener for TcpListener {}
impl Accept for TcpListener {
    type Conn = AddrStream;
    type Error = IoError;
    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        Pin::new(&mut self.get_mut().incoming).poll_accept(cx)
    }
}

pub trait IntoAddrIncoming {
    fn into_incoming(self) -> AddrIncoming;
}

impl IntoAddrIncoming for StdSocketAddr {
    fn into_incoming(self) -> AddrIncoming {
        let mut incoming = AddrIncoming::bind(&self).unwrap();
        incoming.set_nodelay(true);
        incoming
    }
}

impl IntoAddrIncoming for AddrIncoming {
    fn into_incoming(self) -> AddrIncoming {
        self
    }
}

impl<T: ToSocketAddrs + ?Sized> IntoAddrIncoming for &T {
    fn into_incoming(self) -> AddrIncoming {
        for addr in self
            .to_socket_addrs()
            .expect("failed to create AddrIncoming")
        {
            if let Ok(mut incoming) = AddrIncoming::bind(&addr) {
                incoming.set_nodelay(true);
                return incoming;
            }
        }
        panic!("failed to create AddrIncoming");
    }
}

impl<I: Into<IpAddr>> IntoAddrIncoming for (I, u16) {
    fn into_incoming(self) -> AddrIncoming {
        let mut incoming = AddrIncoming::bind(&self.into()).expect("failed to create AddrIncoming");
        incoming.set_nodelay(true);
        incoming
    }
}

#[cfg(test)]
mod tests {
    use futures_util::{Stream, StreamExt};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    use super::*;

    impl Stream for TcpListener {
        type Item = Result<AddrStream, IoError>;
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.poll_accept(cx)
        }
    }

    impl<A, B> Stream for JoinedListener<A, B>
    where
        A: Accept + Send + Unpin + 'static,
        B: Accept + Send + Unpin + 'static,
        A::Conn: Transport,
        B::Conn: Transport,
    {
        type Item = Result<JoinedStream<A::Conn, B::Conn>, IoError>;
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.poll_accept(cx)
        }
    }

    #[tokio::test]
    async fn test_tcp_listener() {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 6878));

        let mut listener = TcpListener::bind(addr);
        tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_i32(150).await.unwrap();
        });

        let mut stream = listener.next().await.unwrap().unwrap();
        assert_eq!(stream.read_i32().await.unwrap(), 150);
    }

    #[tokio::test]
    async fn test_joined_listener() {
        let addr1 = std::net::SocketAddr::from(([127, 0, 0, 1], 6978));
        let addr2 = std::net::SocketAddr::from(([127, 0, 0, 1], 6979));

        let mut listener = TcpListener::bind(addr1).join(TcpListener::bind(addr2));
        tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr1).await.unwrap();
            stream.write_i32(50).await.unwrap();

            let mut stream = TcpStream::connect(addr2).await.unwrap();
            stream.write_i32(100).await.unwrap();
        });
        let mut stream = listener.next().await.unwrap().unwrap();
        let first = stream.read_i32().await.unwrap();
        let mut stream = listener.next().await.unwrap().unwrap();
        let second = stream.read_i32().await.unwrap();
        assert_eq!(first + second, 150);
    }
}
