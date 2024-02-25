use std::{
    io::{Read, Write},
    net::{
        SocketAddr,
        TcpStream
    }
};
use overcast_core::networking::{message::{ClientToServerTcpMessage, Header, MAX_CLIENT_MESSAGE_SIZE}, serialization::Serializable};

const SERVER_RECV_BUFFER_SIZE: usize = Header::MAX_BIN_SIZE + MAX_CLIENT_MESSAGE_SIZE;

/// Client object on the server.
pub(super) struct Client {
    id: u32,
    tcp_stream: TcpStream,
    recv_buffer: [u8; SERVER_RECV_BUFFER_SIZE],
    tcp_addr: SocketAddr,
    player: bevy::prelude::Entity,
}

impl Client {
    pub(crate) fn new(id: u32, tcp_stream: TcpStream, tcp_addr: SocketAddr, player: bevy::prelude::Entity) -> Client {
        Client {
            id,
            tcp_stream,
            recv_buffer: [0u8; SERVER_RECV_BUFFER_SIZE],
            tcp_addr,
            player,
        }
    }

    pub(crate) fn send_tcp(&mut self, message: &[u8]) -> Result<(), std::io::Error> {
        if self.tcp_stream.write(message)? < message.len() {
            // the whole message could not be writen, what do we do ?
            unimplemented!()
        }
        Ok(())
    }

    /// Attempts to read a tcp packet.
    /// TODO : add security, for now we are trusting the client way too much.
    pub(crate) fn incoming_tcp(&mut self) -> Result<Option<ClientToServerTcpMessage>, std::io::Error> {
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
                return Ok(Some(ClientToServerTcpMessage::deserialize(&self.recv_buffer[Header::MAX_BIN_SIZE..])));
            } 
        }
        Ok(None)
    }
}
