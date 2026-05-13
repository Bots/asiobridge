use crate::connection::{Connection, ConnectionType};
use crate::mixer::Mixer;
use crate::network::{ActiveNetworkStream, NetworkStream};
use crate::profile::{Profile, ProfileManager};
use crate::rack::Rack;
use crate::recorder::AudioRecorder;
use crate::resampler::Resampler;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Channel level data with RMS and peak measurements
#[derive(Debug, Clone, Copy)]
pub struct ChannelLevel {
    pub rms: f32,
    pub peak: f32,
}

impl ChannelLevel {
    pub fn new() -> Self {
        Self { rms: 0.0, peak: 0.0 }
    }
}

/// Audio routing engine — manages racks, connections, and the audio pipeline
pub struct AudioEngine {
    racks: HashMap<String, Rack>,
    connections: Vec<Connection>,
    mixer: Mixer,
    resampler: Resampler,
    profile_manager: ProfileManager,
    network_streams: Vec<NetworkStream>,
    active_network_streams: Vec<ActiveNetworkStream>,
    sample_rate: u32,
    bit_depth: u32,
    channels: u16,
    is_running: bool,
    audio_tx: Option<Sender<Vec<f32>>>,
    audio_rx: Option<Receiver<Vec<f32>>>,
    channel_levels: HashMap<(String, u32), ChannelLevel>,
    recorder: AudioRecorder,
}

impl AudioEngine {
    pub fn new(sample_rate: u32, bit_depth: u32, channels: u16) -> Self {
        let (audio_tx, audio_rx) = crossbeam_channel::unbounded();
        let recorder_dir = PathBuf::from("recordings");
        Self {
            racks: HashMap::new(),
            connections: Vec::new(),
            mixer: Mixer::new(channels as usize, sample_rate),
            resampler: Resampler::new(sample_rate, sample_rate, channels as usize),
            profile_manager: ProfileManager::new(),
            network_streams: Vec::new(),
            active_network_streams: Vec::new(),
            sample_rate,
            bit_depth,
            channels,
            is_running: false,
            audio_tx: Some(audio_tx),
            audio_rx: Some(audio_rx),
            channel_levels: HashMap::new(),
            recorder: AudioRecorder::new(recorder_dir, sample_rate, bit_depth as u16, channels),
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
            connection.source_rack,
            connection.source_channel,
            connection.dest_rack,
            connection.dest_channel
        );
        self.connections.push(connection);
    }

    pub fn remove_connection(&mut self, source_rack: &str, source_channel: u32) {
        self.connections
            .retain(|c| !(c.source_rack == source_rack && c.source_channel == source_channel));
    }

    pub fn get_connections(&self) -> &[Connection] {
        &self.connections
    }

    pub fn get_racks(&self) -> &HashMap<String, Rack> {
        &self.racks
    }

    pub fn save_profile(&mut self, slot: usize, name: &str) -> bool {
        let mut profile = Profile::new(name.to_string());

        // Save racks
        for (id, rack) in self.racks.iter() {
            let rack_data = serde_json::json!({
                "id": id,
                "channels": rack.channels
            });
            profile.racks.insert(id.clone(), rack_data);
        }

        // Save connections
        for conn in &self.connections {
            let conn_data = serde_json::json!(conn);
            profile.connections.push(conn_data);
        }

        // Save settings
        profile.global_settings.insert("sample_rate".to_string(), serde_json::json!(self.sample_rate));
        profile.global_settings.insert("bit_depth".to_string(), serde_json::json!(self.bit_depth));
        profile.global_settings.insert("channels".to_string(), serde_json::json!(self.channels));

        self.profile_manager.save(slot, profile)
    }

    pub fn load_profile(&mut self, slot: usize) -> bool {
        match self.profile_manager.load(slot) {
            Some(profile) => {
                // Restore settings
                if let Some(sr) = profile.global_settings.get("sample_rate") {
                    if let Some(rate) = sr.as_u64() {
                        self.sample_rate = rate as u32;
                    }
                }
                if let Some(bd) = profile.global_settings.get("bit_depth") {
                    if let Some(depth) = bd.as_u64() {
                        self.bit_depth = depth as u32;
                    }
                }
                if let Some(ch) = profile.global_settings.get("channels") {
                    if let Some(count) = ch.as_u64() {
                        self.channels = count as u16;
                    }
                }

                info!("Loaded profile '{}' from slot {}", profile.name, slot);
                true
            }
            None => {
                warn!("No profile in slot {}", slot);
                false
            }
        }
    }

    pub fn add_network_stream(&mut self, stream: NetworkStream) {
        info!(
            "Adding network stream: {}:{} ch{} {}Hz {}bit",
            stream.host, stream.port, stream.channels, stream.sample_rate, stream.bit_depth
        );

        // Stop any existing active streams first
        for active in self.active_network_streams.iter_mut() {
            active.stop();
        }
        self.active_network_streams.clear();
        self.network_streams.clear();

        if let Some(active) = stream.start(self.sample_rate, self.channels) {
            self.network_streams.push(stream);
            self.active_network_streams.push(active);
        }
    }

    pub fn get_network_streams(&self) -> &[NetworkStream] {
        &self.network_streams
    }

    pub fn clear_network_streams(&mut self) {
        for active in self.active_network_streams.iter_mut() {
            active.stop();
        }
        self.active_network_streams.clear();
        self.network_streams.clear();
    }

    pub fn get_network_audio(&self) -> Vec<f32> {
        for active in &self.active_network_streams {
            let audio = active.get_audio();
            if !audio.is_empty() {
                return audio;
            }
        }
        Vec::new()
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

    pub fn process_audio(&mut self, input: &[f32]) -> Vec<f32> {
        if !self.is_running {
            return input.to_vec();
        }

        // Get incoming network audio and inject it into the network-in rack
        let network_audio = self.get_network_audio();
        if !network_audio.is_empty() {
            if let Some(rack) = self.racks.get_mut("network-in") {
                let channels_to_fill = std::cmp::min(network_audio.len(), rack.channels.len());
                for i in 0..channels_to_fill {
                    rack.channels[i].level = network_audio[i];
                }
            }
        }

        let active_connections: Vec<_> = self.connections.iter().filter(|c| c.is_active).collect();

        // Decay levels slowly (hold ~500ms then decay)
        for level in self.channel_levels.values_mut() {
            level.rms *= 0.99_f32;
            level.peak *= 0.995_f32;
        }

        if active_connections.is_empty() {
            for (i, &sample) in input.iter().enumerate() {
                let abs = sample.abs();
                let entry = self
                    .channel_levels
                    .entry(("input".to_string(), i as u32))
                    .or_insert_with(ChannelLevel::new);
                entry.peak = entry.peak.max(abs);
                entry.rms = (entry.rms * entry.rms + abs * abs) * 0.5_f32;
                entry.rms = entry.rms.sqrt();
            }
            let mut output = input.to_vec();
            let max = output.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
            if max > 0.95 {
                let gain = 0.95 / max;
                for sample in output.iter_mut() {
                    *sample *= gain;
                }
            }
            return output;
        }

        let num_channels = std::cmp::max(input.len(), self.channels as usize);
        let mut output = vec![0.0f32; num_channels];

        for connection in &active_connections {
            let source_rack = match self.racks.get(&connection.source_rack) {
                Some(rack) => rack,
                None => continue,
            };

            let dest_rack = match self.racks.get(&connection.dest_rack) {
                Some(rack) => rack,
                None => continue,
            };

            let source_ch = connection.source_channel as usize;
            let dest_ch = connection.dest_channel as usize;

            if source_ch >= source_rack.channels.len() {
                continue;
            }

            if dest_ch >= dest_rack.channels.len() {
                continue;
            }

            let source_channel = &source_rack.channels[source_ch];
            let dest_channel = &dest_rack.channels[dest_ch];

            if !source_channel.is_active || !dest_channel.is_active {
                continue;
            }

            let gain = source_channel.level;

            match connection.connection_type {
                ConnectionType::Direct
                | ConnectionType::Network
                | ConnectionType::Wdm
                | ConnectionType::Vst
                | ConnectionType::Midi => {
                    if source_ch < input.len() {
                        let abs = input[source_ch].abs();
                        let entry = self
                            .channel_levels
                            .entry((connection.source_rack.clone(), connection.source_channel))
                            .or_insert_with(ChannelLevel::new);
                        entry.peak = entry.peak.max(abs);
                        entry.rms = (entry.rms * entry.rms + abs * abs) * 0.5_f32;
                        entry.rms = entry.rms.sqrt();
                        output[dest_ch] = output[dest_ch].max(0.0) + input[source_ch] * gain;
                    }
                }
                ConnectionType::Null => {
                    continue;
                }
                ConnectionType::MultiClient => {
                    for i in 0..std::cmp::min(input.len(), num_channels) {
                        let abs = input[i % input.len()].abs();
                        let entry = self
                            .channel_levels
                            .entry((connection.source_rack.clone(), connection.source_channel))
                            .or_insert_with(ChannelLevel::new);
                        entry.peak = entry.peak.max(abs);
                        entry.rms = (entry.rms * entry.rms + abs * abs) * 0.5_f32;
                        entry.rms = entry.rms.sqrt();
                        output[i] += input[i % input.len()] * gain;
                    }
                }
            }
        }

        let max = output.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        if max > 0.95 {
            let gain = 0.95 / max;
            for sample in output.iter_mut() {
                *sample *= gain;
            }
        }

        // Write to recorder if active
        if self.recorder.is_recording() {
            self.recorder.write_samples(&output);
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

    pub fn set_bit_depth(&mut self, bit_depth: u32) {
        self.bit_depth = bit_depth;
        info!("Bit depth changed to {}bit", bit_depth);
    }

    pub fn get_channel_count(&self) -> u16 {
        self.channels
    }

    pub fn get_channel_levels(&self) -> &HashMap<(String, u32), ChannelLevel> {
        &self.channel_levels
    }

    pub fn clear_channel_levels(&mut self) {
        self.channel_levels.clear();
    }

    pub fn start_recording(&mut self) -> bool {
        self.recorder.start_recording()
    }

    pub fn stop_recording(&mut self) -> bool {
        self.recorder.stop_recording()
    }

    pub fn is_recording(&self) -> bool {
        self.recorder.is_recording()
    }

    pub fn get_recording_output_dir(&self) -> PathBuf {
        self.recorder.get_output_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rack::{Channel, ChannelId, RackId};

    fn create_test_engine() -> AudioEngine {
        AudioEngine::new(44100, 24, 2)
    }

    fn create_test_rack(id: RackId, channel_count: usize) -> Rack {
        Rack::new(
            id,
            (0..channel_count)
                .map(|i| Channel {
                    id: ChannelId(i as u32),
                    name: format!("Ch {}", i),
                    sample_rate: 44100,
                    bit_depth: 24,
                    is_active: true,
                    level: 1.0,
                })
                .collect(),
        )
    }

    #[test]
    fn test_engine_create() {
        let engine = create_test_engine();
        assert!(!engine.is_running());
        assert_eq!(engine.get_sample_rate(), 44100);
        assert_eq!(engine.get_bit_depth(), 24);
        assert_eq!(engine.get_channel_count(), 2);
    }

    #[test]
    fn test_engine_start_stop() {
        let mut engine = create_test_engine();
        engine.start();
        assert!(engine.is_running());
        engine.stop();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_engine_double_start() {
        let mut engine = create_test_engine();
        engine.start();
        engine.start(); // Should be no-op
        assert!(engine.is_running());
    }

    #[test]
    fn test_sample_rate_change() {
        let mut engine = create_test_engine();
        engine.set_sample_rate(48000);
        assert_eq!(engine.get_sample_rate(), 48000);
    }

    #[test]
    fn test_bit_depth_change() {
        let mut engine = create_test_engine();
        engine.set_bit_depth(16);
        assert_eq!(engine.get_bit_depth(), 16);
    }

    #[test]
    fn test_process_audio_stopped() {
        let mut engine = create_test_engine();
        let input = vec![0.5f32, 0.3];
        let output = engine.process_audio(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_process_audio_running() {
        let mut engine = create_test_engine();
        engine.start();
        let input = vec![0.5f32, 0.3];
        let output = engine.process_audio(&input);
        assert!(!output.is_empty());
    }

    #[test]
    fn test_level_tracking() {
        let mut engine = create_test_engine();
        engine.start();
        engine.add_rack(create_test_rack(RackId::AsioDriverIn, 2));
        engine.add_rack(create_test_rack(RackId::MixOut, 2));
        engine.add_connection(Connection {
            source_rack: "ASIO Driver IN".to_string(),
            source_channel: 0,
            dest_rack: "Mix OUT".to_string(),
            dest_channel: 0,
            connection_type: ConnectionType::Direct,
            is_active: true,
        });

        // Send a strong signal
        for _ in 0..100 {
            engine.process_audio(&[0.8, 0.5]);
        }

        let levels = engine.get_channel_levels();
        let ch0_level = levels.get(&("ASIO Driver IN".to_string(), 0));
        assert!(ch0_level.is_some(), "Level should be tracked for source channel");
        assert!(ch0_level.unwrap().peak > 0.5, "Peak should be > 0.5");
    }

    #[test]
    fn test_level_decay() {
        let mut engine = create_test_engine();
        engine.start();

        // Send a signal
        engine.process_audio(&[0.9, 0.0]);

        let levels = engine.get_channel_levels();
        let initial_peak = levels.get(&("input".to_string(), 0)).unwrap().peak;
        assert!(initial_peak > 0.5);

        // Send silence and check decay
        for _ in 0..500 {
            engine.process_audio(&[0.0, 0.0]);
        }

        let levels = engine.get_channel_levels();
        let decayed_peak = levels.get(&("input".to_string(), 0)).unwrap().peak;
        assert!(decayed_peak < initial_peak);
    }

    #[test]
    fn test_recording_start_stop() {
        let mut engine = create_test_engine();
        assert!(!engine.is_recording());
        assert!(engine.start_recording());
        assert!(engine.is_recording());
        assert!(engine.stop_recording());
        assert!(!engine.is_recording());
    }

    #[test]
    fn test_recording_double_start() {
        let mut engine = create_test_engine();
        assert!(engine.start_recording());
        assert!(!engine.start_recording()); // Should fail
    }

    #[test]
    fn test_channel_level_new() {
        let level = ChannelLevel::new();
        assert_eq!(level.rms, 0.0);
        assert_eq!(level.peak, 0.0);
    }
}
