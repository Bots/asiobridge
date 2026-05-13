use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use tracing::{error, info};

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

/// Network stream with active sender/receiver threads
pub struct ActiveNetworkStream {
    pub stream: NetworkStream,
    send_handle: Option<JoinHandle<()>>,
    recv_handle: Option<JoinHandle<()>>,
    shared_audio: Arc<Mutex<Vec<f32>>>,
}

impl ActiveNetworkStream {
    pub fn get_audio(&self) -> Vec<f32> {
        match self.shared_audio.lock() {
            Ok(audio) => audio.clone(),
            Err(_) => Vec::new(),
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.send_handle.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.recv_handle.take() {
            let _ = handle.join();
        }
        self.stream.is_active = false;
    }
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

    pub fn start(&self, sample_rate: u32, channels: u16) -> Option<ActiveNetworkStream> {
        let addr = format!("{}:{}", self.host, self.port);
        let shared_audio = Arc::new(Mutex::new(Vec::new()));
        let audio_for_recv = shared_audio.clone();
        let audio_for_send = shared_audio.clone();

        // Start receiver thread
        let recv_handle = {
            let audio_data = audio_for_recv.clone();
            let port = self.port;
            Some(std::thread::spawn(move || {
                let socket = match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
                    Ok(s) => s,
                    Err(e) => {
                        error!("Network recv bind error port {}: {}", port, e);
                        return;
                    }
                };
                socket.set_nonblocking(false).ok();
                info!("Network recv listening on port {}", port);

                let mut buffer = [0u8; 65535];
                loop {
                    match socket.recv(&mut buffer) {
                        Ok(len) => {
                            if len < 4 {
                                continue;
                            }
                            let num_samples =
                                u32::from_le_bytes(buffer[0..4].try_into().unwrap_or([0; 4]))
                                    as usize;
                            if num_samples == 0 || num_samples > 32768 {
                                continue;
                            }
                            let data_len = num_samples * 4;
                            if len < 4 + data_len {
                                continue;
                            }
                            let f32_data: Vec<f32> = buffer[4..4 + data_len]
                                .chunks_exact(4)
                                .map(|c| f32::from_le_bytes(c.try_into().unwrap_or([0; 4])))
                                .collect();
                            if let Ok(mut audio) = audio_data.lock() {
                                *audio = f32_data;
                            }
                        }
                        Err(e) => {
                            error!("Network recv error: {}", e);
                            break;
                        }
                    }
                }
            }))
        };

        // Start sender thread
        let send_handle = {
            let audio_data = audio_for_send.clone();
            let socket = match UdpSocket::bind("0.0.0.0:0") {
                Ok(s) => s,
                Err(e) => {
                    error!("Network send bind error: {}", e);
                    return None;
                }
            };
            if socket.connect(&addr).is_err() {
                error!("Network send connect error to {}", addr);
                return None;
            }
            info!("Network send connecting to {}", addr);

            let sr = self.sample_rate;
            let ch = self.channels;

            Some(std::thread::spawn(move || {
                let mut packet = Vec::with_capacity(65535);
                loop {
                    let data = match audio_data.lock() {
                        Ok(d) => d.clone(),
                        Err(_) => break,
                    };

                    if !data.is_empty() {
                        packet.clear();
                        packet.extend_from_slice(&(data.len() as u32).to_le_bytes());
                        for &sample in &data {
                            packet.extend_from_slice(&sample.to_le_bytes());
                        }
                        if socket.send(&packet).is_err() {
                            break;
                        }
                    }

                    let interval = (1_000_000u64 / sr as u64 / ch as u64).max(100);
                    std::thread::sleep(std::time::Duration::from_micros(interval));
                }
            }))
        };

        Some(ActiveNetworkStream {
            stream: NetworkStream {
                host: self.host.clone(),
                port: self.port,
                sample_rate,
                bit_depth: self.bit_depth,
                channels,
                is_active: true,
            },
            send_handle,
            recv_handle,
            shared_audio,
        })
    }
}
