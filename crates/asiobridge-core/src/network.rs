use serde::{Deserialize, Serialize};

/// UDP network stream for audio between AsioBridge instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStream {
  pub host: String,
  pub port: u16,
  pub sample_rate: u32,
  pub bit_depth: u32,
  pub channels: u16,
  pub is_active: bool,
}

impl NetworkStream {
  pub fn new(host: String, port: u16) -> Self {
    Self {
      host,
      port,
      sample_rate: 44100,
      bit_depth: 24,
      channels: 2,
      is_active: false,
    }
  }
}
