use std::{
    collections::HashMap, io::Write, net::{
        Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket
    }
};
use bevy::prelude::{
    Commands,
    ResMut
};
use overcast_core::{
    log,
    networking::{
        message::{
            Header,
            ServerToClientTcpMessage,
            MAX_SERVER_MESSAGE_SIZE
        },
        serialization::Serializable
    },
};

use crate::config::ServerConfig;

use super::client::Client;

const SERVER_SEND_BUFFER_SIZE: usize = Header::MAX_BIN_SIZE + MAX_SERVER_MESSAGE_SIZE;

/// Networking part of the overcast server.
/// This resource is responsible for listening for incoming connections,
/// receive and route packets.
#[derive(bevy::prelude::Resource)]
pub(crate) struct NetworkManager {
    tcp_listener: TcpListener,
    udp_socket: UdpSocket,
    send_buffer: [u8; SERVER_SEND_BUFFER_SIZE],
    max_player_count: u32,
    next_player_id: u32,
    clients: HashMap<u32, Client>,
    logger: log::Logger,
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        self.send_tcp_to_all(ServerToClientTcpMessage::ServerClosing);
        self.logger.log("Shutting down network", log::LogLevel::Info);
    }
}

impl NetworkManager {
    pub(crate) fn new(logger: log::Logger, config: &ServerConfig) -> Result<NetworkManager, std::io::Error> {
        logger.log(&format!("Creating listener for tcp connections on port {}.", config.tcp_port), log::LogLevel::Info);
        let tcp_listener = TcpListener::bind(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.tcp_port))
        )?;
        tcp_listener.set_nonblocking(true)?;
        
        logger.log(&format!("Creating socket for udp packets on port {}.", config.udp_port), log::LogLevel::Info);
        let udp_socket = UdpSocket::bind(
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.udp_port))
        )?;
        udp_socket.set_nonblocking(true)?;

        Ok(NetworkManager {
            tcp_listener,
            udp_socket,
            send_buffer: [0u8; SERVER_SEND_BUFFER_SIZE],
            max_player_count: config.max_player_count,
            next_player_id: 0,
            clients: HashMap::new(),
            logger,
        })
    }

    fn get_next_id(&mut self) -> u32 {
        let id = self.next_player_id;
        let overflow;
        (self.next_player_id, overflow) = self.next_player_id.overflowing_add(1);
        if overflow {
            self.logger.log("Network client id have overflowed.", log::LogLevel::Warning);
        }
        id
    }

    pub(crate) fn handle_incoming(&mut self, commands: &mut Commands) {
        loop {
            match self.tcp_listener.accept() {
                Ok((mut tcp_stream, tcp_addr)) => {
                    self.logger.log(&format!("Incoming connection from {tcp_addr}"), log::LogLevel::Info);
                    if self.clients.len() < self.max_player_count as usize {
                        let id = self.get_next_id();
                        tcp_stream.set_nonblocking(true);
                        
                        // todo : spawn player in the world
                        let spawn_command = commands.spawn(());
                        
                        let client = Client::new(id, tcp_stream, tcp_addr, spawn_command.id());
                        self.logger.log("Connection accepted, new player joined the game.", log::LogLevel::Info);
                        self.clients.insert(id, client);
                        self.send_tcp_to_client(id, ServerToClientTcpMessage::WelcomeIn);
                    }
                    else {
                        self.logger.log("Server full, rejecting connection.", log::LogLevel::Info);
                        self.load_send_buffer_with_tcp(ServerToClientTcpMessage::ServerFull);
                        if let Err(e) =  tcp_stream.write(&self.send_buffer) {
                            self.logger.log(&format!("Failed to send refused connection message: {e}"), log::LogLevel::Warning);
                        }
                        // tcp stream will be drop, and will get closed here
                    }
                }
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::WouldBlock => break,
                        _ => { /* TODO: handle */}
                    }
                }
            }
        }
    }

    pub(crate) fn handle_messages(&mut self, _commands: &mut Commands) {
        for (id, client) in self.clients.iter_mut() {
            while let Some(tcp_packet) = match client.incoming_tcp() {
                Ok(ok) => ok,
                Err(e) => {
                    self.logger.log(&format!("Error while receiving tcp packet from client {id}: {e}"), log::LogLevel::Warning);
                    None
                }
            } {
                match tcp_packet {


                    #[allow(unreachable_patterns)] // backward compatibility
                    _ => self.logger.log(&format!("Network manager handler not implemented for message {:?}", tcp_packet), log::LogLevel::Warning),
                }
            }
        }
    }

    fn load_send_buffer_with_tcp(&mut self, message: ServerToClientTcpMessage) -> usize {
        // SAFETY: the server send_buffer size is computed at compile time from
        // the messages. If we don't have enough space to serialize, it's a programming error, 
        // so we should panic on it.
        let message_size = message.serialize(&mut self.send_buffer[Header::MAX_BIN_SIZE..]);
        let header = Header {
            client: 0,
            size: message_size as u32,
        };
        let header_size = header.serialize(&mut self.send_buffer);
        header_size + message_size
    }

    fn send_tcp_to_client(&mut self, client_id: u32, message: ServerToClientTcpMessage) {
        let to_send_size = self.load_send_buffer_with_tcp(message);
        match self.clients.get_mut(&client_id) {
            Some(client) => if let Err(e) = client.send_tcp(&self.send_buffer[0..to_send_size]) {
                self.logger.log(&format!("Error while sending tcp message to client: {e}"), log::LogLevel::Warning);
            },
            None => self.logger.log("Unable to send tcp message: client id not found.", log::LogLevel::Warning),
        }
    }

    fn send_tcp_to_all(&mut self, message: ServerToClientTcpMessage) {
        let to_send_size = self.load_send_buffer_with_tcp(message);
        for (_, client) in self.clients.iter_mut() {
            if let Err(e) = client.send_tcp(&self.send_buffer[0..to_send_size]) {
                self.logger.log(&format!("Error while sending tcp message to client: {e}"), log::LogLevel::Warning)
            }
        }
    }


    /// First network manager update: 
    /// Handles incoming players and messages, to be processed duting the current frame.
    /// Therefore, this update should be before any game processing updates. 
    pub(crate) fn recv_update(mut commands: Commands, mut network_manager: ResMut<NetworkManager>) {
        network_manager.handle_incoming(&mut commands);
        network_manager.handle_messages(&mut commands);
    }

    /// Second network manager update:
    /// Send all messages after this frame update.
    /// This should be after any game processing updates.
    pub(crate) fn send_update() {

    }

}
