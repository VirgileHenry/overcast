
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub from_client: u32,
    pub size: u32,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerToClientTcpMessage {
    /// The server received the handshake, and accepted the client in the game.
    WelcomeIn,
    /// The server reach the max allowed amount of clients and is refusing the connection.
    ServerFull,
    /// The server is shuting down, and disconnected the client.
    /// The client is expected to close it's socket.
    ServerClosing,
}


#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerToClientUdpMessage {
    
}



#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientToServerTcpMessage {
    /// First message.
    /// The client wish to connect to the server.
    Handshake,
    /// The client loaded all client side resources,
    /// and is ready to load the world.
    ReadyToLoad,
    /// The client is leaving the game,
    /// and is expecting no further responses.
    Leaving,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientToServerUdpMessage {
    
}
