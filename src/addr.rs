#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum SocketAddr {
    IPv4(std::net::SocketAddrV4),
    IPv6(std::net::SocketAddrV6),
}
impl From<std::net::SocketAddr> for SocketAddr {
    fn from(addr: std::net::SocketAddr) -> Self {
        match addr {
            std::net::SocketAddr::V4(val) => SocketAddr::IPv4(val),
            std::net::SocketAddr::V6(val) => SocketAddr::IPv6(val),
        }
    }
}
impl From<std::net::SocketAddrV4> for SocketAddr {
    fn from(addr: std::net::SocketAddrV4) -> Self {
        SocketAddr::IPv4(addr)
    }
}
impl From<std::net::SocketAddrV6> for SocketAddr {
    fn from(addr: std::net::SocketAddrV6) -> Self {
        SocketAddr::IPv6(addr)
    }
}

impl SocketAddr {
    pub fn is_ipv4(&self) -> bool {
        matches!(*self, SocketAddr::IPv4(_))
    }
    pub fn is_ipv6(&self) -> bool {
        matches!(*self, SocketAddr::IPv6(_))
    }
    pub fn as_ipv4(&self) -> Option<&std::net::SocketAddrV4> {
        match self {
            SocketAddr::IPv4(addr) => Some(addr),
            _ => None,
        }
    }
    pub fn as_ipv6(&self) -> Option<&std::net::SocketAddrV6> {
        match self {
            SocketAddr::IPv6(addr) => Some(addr),
            _ => None,
        }
    }
}

impl std::fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketAddr::IPv4(addr) => write!(f, "socket://{}", addr),
            SocketAddr::IPv6(addr) => write!(f, "socket://{}", addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_addr_ipv4() {
        let ipv4: std::net::SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let ipv4: SocketAddr = ipv4.into();
        assert!(ipv4.is_ipv4());
        assert!(!ipv4.is_ipv6());
    }
    #[tokio::test]
    async fn test_addr_ipv6() {
        let ipv6 = std::net::SocketAddr::new(
            std::net::IpAddr::V6(std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 65535, 0, 1)),
            8080,
        );
        let ipv6: SocketAddr = ipv6.into();
        assert!(ipv6.is_ipv6());
        assert!(!ipv6.is_ipv4());
    }
}
