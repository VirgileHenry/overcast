

// todo : load this struct from a file
pub(crate) struct ServerConfig {
    pub(crate) log_file: String,
    pub(crate) max_player_count: u32,
    pub(crate) udp_port: u16,
    pub(crate) tcp_port: u16,
    pub(crate) tick_rate: f32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            log_file: "logs.log".to_string(),
            max_player_count: 32,
            udp_port: 41671,
            tcp_port: 41672,
            tick_rate: 60.0,
        }
    }
}

impl ServerConfig {
    pub(crate) fn frame_delay(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f32(1.0 / self.tick_rate)
    }
}