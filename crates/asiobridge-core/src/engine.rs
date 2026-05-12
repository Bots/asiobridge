use crate::connection::{Connection, ConnectionType};
use crate::mixer::Mixer;
use crate::network::NetworkStream;
use crate::profile::{Profile, ProfileManager};
use crate::rack::Rack;
use crate::resampler::Resampler;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use tracing::{info, warn};

/// Audio routing engine — manages racks, connections, and the audio pipeline
pub struct AudioEngine {
  racks: HashMap<String, Rack>,
  connections: Vec<Connection>,
  mixer: Mixer,
  resampler: Resampler,
  profile_manager: ProfileManager,
  network_streams: Vec<NetworkStream>,
  sample_rate: u32,
  bit_depth: u32,
  channels: u16,
  is_running: bool,
  audio_tx: Option<Sender<Vec<f32>>>,
  audio_rx: Option<Receiver<Vec<f32>>>,
}

impl AudioEngine {
  pub fn new(sample_rate: u32, bit_depth: u32, channels: u16) -> Self {
    let (audio_tx, audio_rx) = crossbeam_channel::unbounded();
    Self {
      racks: HashMap::new(),
      connections: Vec::new(),
      mixer: Mixer::new(channels as usize, sample_rate),
      resampler: Resampler::new(sample_rate, sample_rate, channels as usize),
      profile_manager: ProfileManager::new(),
      network_streams: Vec::new(),
      sample_rate,
      bit_depth,
      channels,
      is_running: false,
      audio_tx: Some(audio_tx),
      audio_rx: Some(audio_rx),
    }
  }

  pub fn add_rack(&mut self, rack: Rack) {
    info!("Adding rack: {}", rack.id);
    self.racks.insert(rack.id.to_string(), rack);
  }

  pub fn remove_rack(&mut self, rack_id: &str) {
    info!("Removing rack: {}", rack_id);
    self.racks.remove(rack_id);
  }

  pub fn add_connection(&mut self, connection: Connection) {
    info!(
      "Adding connection: {} ch{} -> {} ch{}",
      connection.source_rack, connection.source_channel,
      connection.dest_rack, connection.dest_channel
    );
    self.connections.push(connection);
  }

  pub fn remove_connection(&mut self, source_rack: &str, source_channel: u32) {
    self.connections.retain(|c| !(c.source_rack == source_rack && c.source_channel == source_channel));
  }

  pub fn get_connections(&self) -> &[Connection] {
    &self.connections
  }

  pub fn get_racks(&self) -> &HashMap<String, Rack> {
    &self.racks
  }

  pub fn save_profile(&mut self, slot: usize, name: &str) {
    let profile = Profile::new(name.to_string());
    self.profile_manager.save(slot, profile);
    info!("Saved profile to slot {}", slot);
  }

  pub fn load_profile(&mut self, slot: usize) -> bool {
    if self.profile_manager.load(slot).is_some() {
      info!("Loaded profile from slot {}", slot);
      true
    } else {
      warn!("No profile in slot {}", slot);
      false
    }
  }

  pub fn add_network_stream(&mut self, stream: NetworkStream) {
    info!(
      "Adding network stream: {}:{} ch{} {}Hz {}bit",
      stream.host, stream.port, stream.channels, stream.sample_rate, stream.bit_depth
    );
    self.network_streams.push(stream);
  }

  pub fn get_network_streams(&self) -> &[NetworkStream] {
    &self.network_streams
  }

  pub fn clear_network_streams(&mut self) {
    self.network_streams.clear();
  }

  pub fn start(&mut self) {
    if self.is_running {
      warn!("Audio engine already running");
      return;
    }
    info!(
      "Starting audio engine: {}Hz {}bit {}ch",
      self.sample_rate, self.bit_depth, self.channels
    );
    self.is_running = true;
  }

  pub fn stop(&mut self) {
    if !self.is_running {
      return;
    }
    info!("Stopping audio engine");
    self.is_running = false;
  }

  pub fn is_running(&self) -> bool {
    self.is_running
  }

  pub fn process_audio(&self, input: &[f32]) -> Vec<f32> {
    if !self.is_running {
      return input.to_vec();
    }

    let num_channels = input.len();
    let mut output = vec![0.0f32; num_channels];

    for connection in &self.connections {
      if !connection.is_active {
        continue;
      }

      let source_rack = match self.racks.get(&connection.source_rack) {
        Some(rack) => rack,
        None => {
          warn!("Source rack not found: {}", connection.source_rack);
          continue;
        }
      };

      let dest_rack = match self.racks.get(&connection.dest_rack) {
        Some(rack) => rack,
        None => {
          warn!("Dest rack not found: {}", connection.dest_rack);
          continue;
        }
      };

      let source_ch = connection.source_channel as usize;
      let dest_ch = connection.dest_channel as usize;

      if source_ch >= source_rack.channels.len() {
        warn!(
          "Source channel {} out of range for rack {}",
          source_ch, connection.source_rack
        );
        continue;
      }

      if dest_ch >= dest_rack.channels.len() {
        warn!(
          "Dest channel {} out of range for rack {}",
          dest_ch, connection.dest_rack
        );
        continue;
      }

      let source_channel = &source_rack.channels[source_ch];
      let dest_channel = &dest_rack.channels[dest_ch];

      if !source_channel.is_active || !dest_channel.is_active {
        continue;
      }

      let gain = source_channel.level;

      match connection.connection_type {
        ConnectionType::Direct => {
          if source_ch < input.len() {
            output[dest_ch] = input[source_ch] * gain;
          }
        }
        ConnectionType::Network => {
          if source_ch < input.len() {
            output[dest_ch] = input[source_ch] * gain;
          }
        }
        ConnectionType::Wdm => {
          if source_ch < input.len() {
            output[dest_ch] = input[source_ch] * gain;
          }
        }
        ConnectionType::Null => {
          continue;
        }
        ConnectionType::MultiClient => {
          for i in 0..std::cmp::min(input.len(), num_channels) {
            output[i] += input[i % input.len()] * gain;
          }
        }
        ConnectionType::Vst | ConnectionType::Midi => {
          if source_ch < input.len() {
            output[dest_ch] = input[source_ch] * gain;
          }
        }
      }
    }

    output
  }

  pub fn get_audio_sender(&self) -> Option<Sender<Vec<f32>>> {
    self.audio_tx.clone()
  }

  pub fn get_audio_receiver(&self) -> Option<Receiver<Vec<f32>>> {
    self.audio_rx.clone()
  }

  pub fn set_sample_rate(&mut self, sample_rate: u32) {
    self.sample_rate = sample_rate;
    self.resampler = Resampler::new(sample_rate, sample_rate, self.channels as usize);
    self.mixer = Mixer::new(self.channels as usize, sample_rate);
    info!("Sample rate changed to {}Hz", sample_rate);
  }

  pub fn get_sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub fn get_bit_depth(&self) -> u32 {
    self.bit_depth
  }

  pub fn get_channel_count(&self) -> u16 {
    self.channels
  }
}
