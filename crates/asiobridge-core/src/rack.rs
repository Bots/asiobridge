use serde::{Deserialize, Serialize};

/// Unique identifier for a rack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RackId {
    AsioDriverIn,
    AsioDriverOutMix,
    AsioHostInMix,
    NetworkIn,
    NetworkOut,
    LooperIn,
    LooperOut,
    WdmIn,
    MixOut,
}

/// Unique identifier for a channel within a rack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u32);

impl std::fmt::Display for RackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RackId::AsioDriverIn => write!(f, "ASIO Driver IN"),
            RackId::AsioDriverOutMix => write!(f, "ASIO Driver OUT/MIX"),
            RackId::AsioHostInMix => write!(f, "ASIO Host IN/MIX"),
            RackId::NetworkIn => write!(f, "Network IN"),
            RackId::NetworkOut => write!(f, "Network OUT"),
            RackId::LooperIn => write!(f, "Looper IN"),
            RackId::LooperOut => write!(f, "Looper OUT"),
            RackId::WdmIn => write!(f, "WDM IN"),
            RackId::MixOut => write!(f, "Mix OUT"),
        }
    }
}

/// A rack is a processing unit with input/output channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rack {
    pub id: RackId,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub sample_rate: u32,
    pub bit_depth: u32,
    pub is_active: bool,
    pub level: f32,
}

impl Rack {
    pub fn new(id: RackId, channels: Vec<Channel>) -> Self {
        Self { id, channels }
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }
}
