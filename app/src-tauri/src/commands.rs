use crate::audio_manager::{AudioCommand, AudioManagerHandle};
use asiobridge_core::{AudioEngine, Connection, ConnectionType, RackId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelState {
    pub id: u32,
    pub name: String,
    pub active: bool,
    pub level: f32,
    pub peak: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RackState {
    pub id: String,
    pub name: String,
    pub channels: Vec<ChannelState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionState {
    pub source_rack: String,
    pub source_channel: u32,
    pub dest_rack: String,
    pub dest_channel: u32,
    pub connection_type: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineConfig {
    pub sample_rate: u32,
    pub bit_depth: u32,
    pub channels: u16,
}

#[tauri::command]
pub fn get_racks(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<Vec<RackState>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    let racks = engine.get_racks();
    let levels = engine.get_channel_levels();
    let states: Vec<RackState> = racks
        .values()
        .map(|rack| {
            let rack_id = rack.id.to_string();
            let channels: Vec<ChannelState> = rack
                .channels
                .iter()
                .enumerate()
                .map(|(i, ch)| {
                    let cl = levels
                        .get(&(rack_id.clone(), i as u32))
                        .copied()
                        .unwrap_or(asiobridge_core::ChannelLevel::new());
                    ChannelState {
                        id: ch.id.0,
                        name: ch.name.clone(),
                        active: ch.is_active,
                        level: cl.rms,
                        peak: cl.peak,
                    }
                })
                .collect();
            RackState {
                id: rack_id.clone(),
                name: rack_id,
                channels,
            }
        })
        .collect();
    Ok(states)
}

#[tauri::command]
pub fn get_connections(
    engine: State<Arc<Mutex<AudioEngine>>>,
) -> Result<Vec<ConnectionState>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    let connections = engine.get_connections();
    let states: Vec<ConnectionState> = connections
        .iter()
        .map(|c| ConnectionState {
            source_rack: c.source_rack.clone(),
            source_channel: c.source_channel,
            dest_rack: c.dest_rack.clone(),
            dest_channel: c.dest_channel,
            connection_type: c.connection_type.to_string(),
            is_active: c.is_active,
        })
        .collect();
    Ok(states)
}

#[tauri::command]
pub fn start_engine(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.start();
    Ok(engine.is_running())
}

#[tauri::command]
pub fn stop_engine(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.stop();
    Ok(!engine.is_running())
}

#[tauri::command]
pub fn get_engine_config(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<EngineConfig, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(EngineConfig {
        sample_rate: engine.get_sample_rate(),
        bit_depth: engine.get_bit_depth(),
        channels: engine.get_channel_count(),
    })
}

#[tauri::command]
pub fn set_sample_rate(
    engine: State<Arc<Mutex<AudioEngine>>>,
    sample_rate: u32,
) -> Result<u32, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.set_sample_rate(sample_rate);
    Ok(engine.get_sample_rate())
}

#[tauri::command]
pub fn save_profile(
    engine: State<Arc<Mutex<AudioEngine>>>,
    slot: usize,
    name: String,
) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.save_profile(slot, &name);
    Ok(true)
}

#[tauri::command]
pub fn load_profile(engine: State<Arc<Mutex<AudioEngine>>>, slot: usize) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.load_profile(slot))
}

#[tauri::command]
pub fn add_connection(
    engine: State<Arc<Mutex<AudioEngine>>>,
    source_rack: String,
    source_channel: u32,
    dest_rack: String,
    dest_channel: u32,
    connection_type: String,
) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    let conn_type = match connection_type.as_str() {
        "Direct" => ConnectionType::Direct,
        "Network" => ConnectionType::Network,
        "Wdm" => ConnectionType::Wdm,
        "Null" => ConnectionType::Null,
        "MultiClient" => ConnectionType::MultiClient,
        "Vst" => ConnectionType::Vst,
        "Midi" => ConnectionType::Midi,
        _ => ConnectionType::Direct,
    };
    let connection = Connection {
        source_rack,
        source_channel,
        dest_rack,
        dest_channel,
        connection_type: conn_type,
        is_active: true,
    };
    engine.add_connection(connection);
    Ok(true)
}

#[tauri::command]
pub fn remove_connection(
    engine: State<Arc<Mutex<AudioEngine>>>,
    source_rack: String,
    source_channel: u32,
) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.remove_connection(&source_rack, source_channel);
    Ok(true)
}

#[tauri::command]
pub fn toggle_connection(
    engine: State<Arc<Mutex<AudioEngine>>>,
    source_rack: String,
    source_channel: u32,
    dest_rack: String,
    dest_channel: u32,
) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    let connection = engine
        .get_connections()
        .iter()
        .find(|c| {
            c.source_rack == source_rack
                && c.source_channel == source_channel
                && c.dest_rack == dest_rack
                && c.dest_channel == dest_channel
        })
        .cloned();
    if let Some(conn) = connection {
        let mut updated = conn.clone();
        updated.is_active = !updated.is_active;
        engine.remove_connection(&source_rack, source_channel);
        engine.add_connection(updated);
    }
    Ok(true)
}

#[tauri::command]
pub fn set_bit_depth(
    engine: State<Arc<Mutex<AudioEngine>>>,
    bit_depth: u32,
) -> Result<u32, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.set_bit_depth(bit_depth);
    Ok(engine.get_bit_depth())
}

#[tauri::command]
pub fn get_input_devices(device: State<AudioManagerHandle>) -> Result<Vec<String>, String> {
    Ok(device.get_input_devices_sync())
}

#[tauri::command]
pub fn get_output_devices(device: State<AudioManagerHandle>) -> Result<Vec<String>, String> {
    Ok(device.get_output_devices_sync())
}

#[tauri::command]
pub fn get_default_input(device: State<AudioManagerHandle>) -> Result<Option<String>, String> {
    Ok(device.get_default_input_sync())
}

#[tauri::command]
pub fn get_default_output(device: State<AudioManagerHandle>) -> Result<Option<String>, String> {
    Ok(device.get_default_output_sync())
}

#[tauri::command]
pub fn get_available_drivers(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<Vec<String>, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    let racks = engine.get_racks();
    let drivers: Vec<String> = racks
        .values()
        .filter(|r| {
            r.id == RackId::AsioDriverIn
                || r.id == RackId::AsioDriverOutMix
                || r.id == RackId::AsioHostInMix
        })
        .map(|r| r.id.to_string())
        .collect();
    Ok(drivers)
}

#[tauri::command]
pub fn start_input_device(
    device: State<AudioManagerHandle>,
    device_name: String,
) -> Result<String, String> {
    device
        .send_command(AudioCommand::StartInput(device_name))
        .map(|_| "Input started".to_string())
}

#[tauri::command]
pub fn start_output_device(
    device: State<AudioManagerHandle>,
    device_name: String,
) -> Result<String, String> {
    device
        .send_command(AudioCommand::StartOutput(device_name))
        .map(|_| "Output started".to_string())
}

#[tauri::command]
pub fn stop_audio_device(device: State<AudioManagerHandle>) -> Result<String, String> {
    device
        .send_command(AudioCommand::Stop)
        .map(|_| "Audio stopped".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineStatus {
    pub is_running: bool,
    pub sample_rate: u32,
    pub bit_depth: u32,
    pub channels: u16,
}

#[tauri::command]
pub fn get_engine_status(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<EngineStatus, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(EngineStatus {
        is_running: engine.is_running(),
        sample_rate: engine.get_sample_rate(),
        bit_depth: engine.get_bit_depth(),
        channels: engine.get_channel_count(),
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkStreamConfig {
    pub host: String,
    pub port: u16,
    pub is_active: bool,
}

#[tauri::command]
pub fn start_network_stream(
    engine: State<Arc<Mutex<AudioEngine>>>,
    host: String,
    port: u16,
) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    let stream = asiobridge_core::NetworkStream::new(host, port);
    engine.add_network_stream(stream);
    Ok(true)
}

#[tauri::command]
pub fn stop_network_stream(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    engine.clear_network_streams();
    Ok(true)
}

#[tauri::command]
pub fn get_network_stream_config(
    engine: State<Arc<Mutex<AudioEngine>>>,
) -> Result<NetworkStreamConfig, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    let streams = engine.get_network_streams();
    let stream = streams
        .first()
        .cloned()
        .unwrap_or_else(|| asiobridge_core::NetworkStream::new("127.0.0.1".to_string(), 6997));
    Ok(NetworkStreamConfig {
        host: stream.host,
        port: stream.port,
        is_active: stream.is_active,
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub output_dir: String,
}

#[tauri::command]
pub fn start_recording(
    engine: State<Arc<Mutex<AudioEngine>>>,
    output_dir: String,
) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    let dir = PathBuf::from(output_dir);
    std::fs::create_dir_all(&dir).ok();
    Ok(engine.start_recording())
}

#[tauri::command]
pub fn stop_recording(engine: State<Arc<Mutex<AudioEngine>>>) -> Result<bool, String> {
    let mut engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(engine.stop_recording())
}

#[tauri::command]
pub fn get_recording_status(
    engine: State<Arc<Mutex<AudioEngine>>>,
) -> Result<RecordingStatus, String> {
    let engine = engine.lock().map_err(|e| e.to_string())?;
    Ok(RecordingStatus {
        is_recording: engine.is_recording(),
        output_dir: engine
            .get_recording_output_dir()
            .to_string_lossy()
            .to_string(),
    })
}
