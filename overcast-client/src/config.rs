use std::net::Ipv4Addr;





pub(crate) struct ClientConfig {
    pub(crate) server_ip_addr: Ipv4Addr,
    pub(crate) server_tcp_port: u16,
    pub(crate) server_udp_port: u16,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            server_ip_addr: Ipv4Addr::new(127, 0, 0, 1),
            server_tcp_port: 41671,
            server_udp_port: 41672,
        }
    }
}