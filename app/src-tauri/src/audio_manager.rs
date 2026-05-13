use asiobridge_core::AudioEngine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use ringbuf::{traits::*, HeapRb};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Audio commands that can be sent to the audio thread
pub enum AudioCommand {
    StartInput(String),
    StartOutput(String),
    Stop,
}

/// Parameters for starting audio input
struct InputParams {
    device_name: String,
    channel_count: u16,
    sample_rate: u32,
    buffer_size: usize,
    input_rb: Arc<HeapRb<f32>>,
    engine: Arc<Mutex<AudioEngine>>,
    output_rb: Arc<HeapRb<f32>>,
}

/// Audio manager runs on a dedicated thread and handles all cpal audio I/O
#[allow(dead_code)]
pub struct AudioManagerHandle {
    tx: std::sync::mpsc::Sender<AudioCommand>,
    input_rb: Arc<HeapRb<f32>>,
    output_rb: Arc<HeapRb<f32>>,
    _handle: std::thread::JoinHandle<()>,
}

impl AudioManagerHandle {
    pub fn new(
        engine: Arc<Mutex<AudioEngine>>,
        channel_count: u16,
        sample_rate: u32,
        buffer_size: usize,
    ) -> Self {
        let (tx, rx) = channel();
        let input_rb = Arc::new(HeapRb::new(buffer_size * channel_count as usize * 4));
        let output_rb = Arc::new(HeapRb::new(buffer_size * channel_count as usize * 4));
        let thread_input_rb = input_rb.clone();
        let thread_output_rb = output_rb.clone();

        let handle = std::thread::spawn(move || {
            let mut input_stream: Option<Stream> = None;
            let mut output_stream: Option<Stream> = None;
            let rb_input_rb = thread_input_rb;
            let rb_output_rb = thread_output_rb;

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::StartInput(device_name) => {
                        let params = InputParams {
                            device_name,
                            channel_count,
                            sample_rate,
                            buffer_size,
                            input_rb: rb_input_rb.clone(),
                            engine: engine.clone(),
                            output_rb: rb_output_rb.clone(),
                        };
                        if let Err(e) = Self::start_input(&mut input_stream, params) {
                            error!("Input error: {}", e);
                        }
                    }
                    AudioCommand::StartOutput(device_name) => {
                        if let Err(e) = Self::start_output(
                            &mut output_stream,
                            &device_name,
                            channel_count,
                            sample_rate,
                            buffer_size,
                            rb_output_rb.clone(),
                        ) {
                            error!("Output error: {}", e);
                        }
                    }
                    AudioCommand::Stop => {
                        if let Some(stream) = input_stream.take() {
                            let _ = stream.pause();
                        }
                        if let Some(stream) = output_stream.take() {
                            let _ = stream.pause();
                        }
                    }
                }
            }
        });

        Self {
            tx,
            input_rb,
            output_rb,
            _handle: handle,
        }
    }

    fn start_input(
        stream: &mut Option<Stream>,
        params: InputParams,
    ) -> Result<(), String> {
        let InputParams {
            device_name,
            channel_count,
            sample_rate,
            buffer_size,
            input_rb,
            engine,
            output_rb,
        } = params;

        let host = cpal::default_host();
        let device = Self::find_device(&host, &device_name)
            .ok_or_else(|| format!("Device not found: {}", device_name))?;

        let input_config = device
            .default_input_config()
            .map_err(|e| format!("Failed to get default config: {}", e))?;

        let actual_channels = std::cmp::max(input_config.channels(), channel_count);

        let stream_config = StreamConfig {
            channels: actual_channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
        };

        info!(
            "Starting input: {} ({} ch, {} Hz, {:?})",
            device_name,
            actual_channels,
            sample_rate,
            input_config.sample_format()
        );

        let input_rb = input_rb.clone();
        let engine = engine.clone();
        let output_rb = output_rb.clone();

        let sample_format = input_config.sample_format();

        let new_stream = if sample_format == SampleFormat::F32 {
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| {
                    let (mut prod, _) = input_rb.clone().split();
                    for &sample in data {
                        let _ = prod.try_push(sample);
                    }
                    drop(prod);

                    let processed = if let Ok(mut engine_lock) = engine.lock() {
                        engine_lock.process_audio(data)
                    } else {
                        data.to_vec()
                    };

                    let (mut prod, _) = output_rb.clone().split();
                    for &sample in &processed {
                        let _ = prod.try_push(sample);
                    }
                    drop(prod);
                },
                move |err| {
                    use cpal::StreamError;
                    match err {
                        StreamError::DeviceNotAvailable => {
                            info!("Audio device not available, will retry");
                        }
                        e => {
                            error!("Input stream error: {}", e);
                        }
                    }
                },
                None,
            )
        } else {
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| {
                    let (mut prod, _) = input_rb.clone().split();
                    for &sample in data {
                        let normalized = sample as f32 / 32768.0;
                        let _ = prod.try_push(normalized);
                    }
                    drop(prod);

                    let f32_data: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                    let processed = if let Ok(mut engine_lock) = engine.lock() {
                        engine_lock.process_audio(&f32_data)
                    } else {
                        f32_data
                    };

                    let (mut prod, _) = output_rb.clone().split();
                    for &sample in &processed {
                        let _ = prod.try_push(sample);
                    }
                    drop(prod);
                },
                move |err| {
                    use cpal::StreamError;
                    match err {
                        StreamError::DeviceNotAvailable => {
                            info!("Audio device not available, will retry");
                        }
                        e => {
                            error!("Input stream error: {}", e);
                        }
                    }
                },
                None,
            )
        };

        let new_stream = new_stream.map_err(|e| format!("Failed to build input stream: {}", e))?;
        new_stream
            .play()
            .map_err(|e| format!("Failed to play: {}", e))?;

        if let Some(old_stream) = stream.take() {
            let _ = old_stream.pause();
        }

        *stream = Some(new_stream);
        Ok(())
    }

    fn start_output(
        stream: &mut Option<Stream>,
        device_name: &str,
        channel_count: u16,
        sample_rate: u32,
        buffer_size: usize,
        output_rb: Arc<HeapRb<f32>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();
        let device = Self::find_device(&host, device_name)
            .ok_or_else(|| format!("Device not found: {}", device_name))?;

        let output_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default config: {}", e))?;

        let actual_channels = std::cmp::max(output_config.channels(), channel_count);

        let stream_config = StreamConfig {
            channels: actual_channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
        };

        info!(
            "Starting output: {} ({} ch, {} Hz, {:?})",
            device_name,
            actual_channels,
            sample_rate,
            output_config.sample_format()
        );

        let output_rb = output_rb.clone();
        let sample_format = output_config.sample_format();

        let new_stream = if sample_format == SampleFormat::F32 {
            device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &_| {
                    let (_, mut cons) = output_rb.clone().split();
                    for sample in data.iter_mut() {
                        *sample = cons.try_pop().unwrap_or(0.0);
                    }
                    drop(cons);
                },
                move |err| {
                    use cpal::StreamError;
                    match err {
                        StreamError::DeviceNotAvailable => {
                            info!("Audio device not available, will retry");
                        }
                        e => {
                            error!("Output stream error: {}", e);
                        }
                    }
                },
                None,
            )
        } else {
            device.build_output_stream(
                &stream_config,
                move |data: &mut [i16], _: &_| {
                    let (_, mut cons) = output_rb.clone().split();
                    for sample in data.iter_mut() {
                        let f32_sample = cons.try_pop().unwrap_or(0.0);
                        *sample = (f32_sample * 32767.0) as i16;
                    }
                    drop(cons);
                },
                move |err| {
                    use cpal::StreamError;
                    match err {
                        StreamError::DeviceNotAvailable => {
                            info!("Audio device not available, will retry");
                        }
                        e => {
                            error!("Output stream error: {}", e);
                        }
                    }
                },
                None,
            )
        };

        let new_stream = new_stream.map_err(|e| format!("Failed to build output stream: {}", e))?;
        new_stream
            .play()
            .map_err(|e| format!("Failed to play: {}", e))?;

        if let Some(old_stream) = stream.take() {
            let _ = old_stream.pause();
        }

        *stream = Some(new_stream);
        Ok(())
    }

    fn find_device(host: &cpal::Host, name: &str) -> Option<cpal::Device> {
        host.devices().ok()?.find(|d| {
            d.name()
                .ok()
                .as_ref()
                .map(|n| n == name || name.ends_with(n))
                .unwrap_or(false)
        })
    }

    pub fn send_command(&self, cmd: AudioCommand) -> Result<(), String> {
        self.tx.send(cmd).map_err(|e| e.to_string())
    }

    pub fn get_input_devices_sync(&self) -> Vec<String> {
        let host = cpal::default_host();
        match host.input_devices() {
            Ok(devices) => devices
                .filter_map(|d| d.name().ok().map(|name| format!("Input: {}", name)))
                .collect(),
            Err(e) => {
                error!("Failed to get input devices: {}", e);
                Vec::new()
            }
        }
    }

    pub fn get_output_devices_sync(&self) -> Vec<String> {
        let host = cpal::default_host();
        match host.output_devices() {
            Ok(devices) => devices
                .filter_map(|d| d.name().ok().map(|name| format!("Output: {}", name)))
                .collect(),
            Err(e) => {
                error!("Failed to get output devices: {}", e);
                Vec::new()
            }
        }
    }

    pub fn get_default_input_sync(&self) -> Option<String> {
        let host = cpal::default_host();
        host.default_input_device()
            .and_then(|d| d.name().ok())
            .map(|n| format!("Input: {}", n))
    }

    pub fn get_default_output_sync(&self) -> Option<String> {
        let host = cpal::default_host();
        host.default_output_device()
            .and_then(|d| d.name().ok())
            .map(|n| format!("Output: {}", n))
    }
}
