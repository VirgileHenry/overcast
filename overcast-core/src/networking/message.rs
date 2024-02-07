



#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerToClientTcpMessage {
    /// The server received the handshake, and accepted the client in the game.
    WelcomeIn,
    /// The server disconnected the client,
    /// and is expecting no further responses.
    GetOut,
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
