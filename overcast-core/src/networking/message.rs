use super::serialization::Serializable;

const fn const_max(a: usize, b: usize) -> usize {
    if a > b { a } else { b }
}


pub const HEADER_SERIALIZED_SIZE: usize = Header::MAX_BIN_SIZE;
pub const MAX_SERVER_MESSAGE_SIZE: usize = const_max(ServerToClientTcpMessage::MAX_BIN_SIZE, ServerToClientUdpMessage::MAX_BIN_SIZE); // todo : programatically

#[derive(overcast_macros::Serializable)]
pub struct Header {
    pub client: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
#[derive(overcast_macros::Serializable)]
pub enum ServerToClientTcpMessage {
    /// The server received the handshake, and accepted the client in the game.
    WelcomeIn,
    /// The server reach the max allowed amount of clients and is refusing the connection.
    ServerFull,
    /// The server is shuting down, and disconnected the client.
    /// The client is expected to close it's socket.
    ServerClosing,
}


#[derive(Debug, Clone)]
#[derive(overcast_macros::Serializable)]
pub enum ServerToClientUdpMessage {
    
}


#[derive(Debug, Clone)]
#[derive(overcast_macros::Serializable)]
pub enum ClientToServerTcpMessage {

}

#[derive(Debug, Clone)]
#[derive(overcast_macros::Serializable)]
pub enum ClientToServerUdpMessage {
    
}
