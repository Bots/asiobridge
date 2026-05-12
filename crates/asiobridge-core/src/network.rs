use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

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

  pub fn start_send(&self, audio_data: Arc<Mutex<Vec<f32>>>) -> Option<std::thread::JoinHandle<()>> {
    let addr = format!("{}:{}", self.host, self.port);
    let sample_rate = self.sample_rate;
    let channels = self.channels;

    let socket = match UdpSocket::bind("0.0.0.0:0") {
      Ok(s) => s,
      Err(e) => {
        error!("Failed to bind UDP socket for sending: {}", e);
        return None;
      }
    };

    match socket.connect(&addr) {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to connect UDP socket to {}: {}", addr, e);
        return None;
      }
    }

    info!("Starting network send to {}", addr);

    let audio_data = audio_data.clone();
    Some(std::thread::spawn(move || {
      loop {
        let data = match audio_data.lock() {
          Ok(data) => data.clone(),
          Err(e) => {
            error!("Failed to lock audio data: {}", e);
            break;
          }
        };

        if data.is_empty() {
          std::thread::sleep(std::time::Duration::from_micros(100));
          continue;
        }

        match socket.send(&bincode::serialize(&data).unwrap_or_default()) {
          Ok(_) => {}
          Err(e) => {
            error!("Failed to send audio data: {}", e);
            break;
          }
        }

        std::thread::sleep(std::time::Duration::from_micros((1000000 / sample_rate / channels as u32) as u64));
      }
    }))
  }

  pub fn start_receive(&self, audio_data: Arc<Mutex<Vec<f32>>>) -> Option<std::thread::JoinHandle<()>> {
    let addr = format!("{}:{}", self.host, self.port);
    let socket = match UdpSocket::bind(&addr) {
      Ok(s) => s,
      Err(e) => {
        error!("Failed to bind UDP socket for receiving on {}: {}", addr, e);
        return None;
      }
    };

    info!("Starting network receive on {}", addr);

    let audio_data = audio_data.clone();
    Some(std::thread::spawn(move || {
      let mut buffer = [0u8; 65535];
      loop {
        match socket.recv(&mut buffer) {
          Ok(len) => {
            match bincode::deserialize::<Vec<f32>>(&buffer[..len]) {
              Ok(data) => {
                if let Ok(mut audio) = audio_data.lock() {
                  *audio = data;
                }
              }
              Err(e) => {
                warn!("Failed to deserialize audio data: {}", e);
              }
            }
          }
          Err(e) => {
            error!("Failed to receive audio data: {}", e);
            break;
          }
        }
      }
    }))
  }
}
