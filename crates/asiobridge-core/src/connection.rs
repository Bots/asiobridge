use serde::{Deserialize, Serialize};

/// Types of audio connections between sources and destinations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    Direct,
    Network,
    Wdm,
    Null,
    MultiClient,
    Vst,
    Midi,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionType::Direct => write!(f, "Direct"),
            ConnectionType::Network => write!(f, "Network"),
            ConnectionType::Wdm => write!(f, "WDM"),
            ConnectionType::Null => write!(f, "Null"),
            ConnectionType::MultiClient => write!(f, "Multi-Client"),
            ConnectionType::Vst => write!(f, "VST"),
            ConnectionType::Midi => write!(f, "MIDI"),
        }
    }
}

/// A connection routes audio from a source to a destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub source_rack: String,
    pub source_channel: u32,
    pub dest_rack: String,
    pub dest_channel: u32,
    pub connection_type: ConnectionType,
    pub is_active: bool,
}
