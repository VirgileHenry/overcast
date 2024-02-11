pub mod message;

/// Overcast endian config. Big endian (similar to network endian)
pub const OVERCAST_ENDIAN: serde_binary::binary_stream::Endian = serde_binary::binary_stream::Endian::Big;