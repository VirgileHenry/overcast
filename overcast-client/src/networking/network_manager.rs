use std::{io::Read, net::{SocketAddrV4, TcpStream, UdpSocket}};
use overcast_core::{log::{LogLevel, Logger}, networking::{
    message::{
        Header, ServerToClientTcpMessage, MAX_CLIENT_MESSAGE_SIZE, MAX_SERVER_MESSAGE_SIZE
    },
    serialization::Serializable
}};

use crate::config::ClientConfig;

const SEND_BUFFER_SIZE: usize = Header::MAX_BIN_SIZE + MAX_CLIENT_MESSAGE_SIZE;
const RECV_BUFFER_SIZE: usize = Header::MAX_BIN_SIZE + MAX_SERVER_MESSAGE_SIZE;

#[derive(bevy::prelude::Resource)]
pub(crate) struct NetworkManager {
    tcp_stream: TcpStream,
    udp_socket: UdpSocket,
    send_buffer: [u8; SEND_BUFFER_SIZE],
    recv_buffer: [u8; RECV_BUFFER_SIZE],
    logger: Logger,
}

impl NetworkManager {
    pub(crate) fn new(config: &ClientConfig, logger: Logger) -> Result<NetworkManager, std::io::Error> {
        let server_tcp_addr = SocketAddrV4::new(config.server_ip_addr, config.server_tcp_port);
        logger.log(&format!("Attempting to connect to server with tcp at {server_tcp_addr}"), LogLevel::Info);
        let tcp_stream = TcpStream::connect(server_tcp_addr)?;
        
        let server_udp_addr = SocketAddrV4::new(config.server_ip_addr, config.server_udp_port);
        logger.log(&format!("Attempting to connect to server with udp at {server_udp_addr}"), LogLevel::Info);
        let udp_socket = UdpSocket::bind(server_udp_addr)?;
        udp_socket.connect(server_udp_addr)?;

        Ok(NetworkManager {
            tcp_stream,
            udp_socket,
            send_buffer: [0u8; SEND_BUFFER_SIZE],
            recv_buffer: [0u8; RECV_BUFFER_SIZE],
            logger,
        })
    }

    fn incoming_tcp(&mut self) -> Result<Option<ServerToClientTcpMessage>, std::io::Error> {
        // read guarantees to return a value between 0 and buffer length,
        // so here a value between 0 and BUF_SIZE - cursor, so cursor + return value 
        // can't overflow BUF_SIZE.
        let read = self.tcp_stream.peek(&mut self.recv_buffer)?;
        if read > Header::MAX_BIN_SIZE {
            let header = Header::deserialize(&self.recv_buffer);
            let to_read_size = Header::MAX_BIN_SIZE + header.size as usize;
            if read >= to_read_size {
                // read the exact bytes from the message
                self.tcp_stream.read_exact(&mut self.recv_buffer[..to_read_size])?;
                return Ok(Some(ServerToClientTcpMessage::deserialize(&self.recv_buffer[Header::MAX_BIN_SIZE..])));
            } 
        }
        Ok(None)
    }

    fn handle_messages(&mut self, commands: &mut bevy::prelude::Commands) {
        while let Some(tcp_packet) = match self.incoming_tcp() {
            Ok(tcp_packet) => tcp_packet,
            Err(e) => {
                self.logger.log(&format!("Error while receiving tcp packe: {e}"), LogLevel::Warning);
                None
            }
        } {
            match tcp_packet {



                #[allow(unreachable_patterns)] // backward compatibility
                _ => self.logger.log(&format!("Network manager handler not implemented for message {:?}", tcp_packet), LogLevel::Warning),
            }
        }
    }

    pub(crate) fn recv_update(mut commands: bevy::prelude::Commands, mut network_manager: bevy::prelude::ResMut<NetworkManager>) {
        network_manager.handle_messages(&mut commands);
    }
}