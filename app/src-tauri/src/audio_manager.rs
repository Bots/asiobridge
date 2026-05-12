use asiobridge_core::AudioEngine;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use ringbuf::{HeapRb, traits::*};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Audio commands that can be sent to the audio thread
pub enum AudioCommand {
  StartInput(String),
  StartOutput(String),
  Stop,
  WriteOutput(Vec<f32>),
}

/// Audio manager runs on a dedicated thread and handles all cpal audio I/O
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
    let input_rb = Arc::new(HeapRb::new(buffer_size * channel_count as usize));
    let output_rb = Arc::new(HeapRb::new(buffer_size * channel_count as usize));
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
            if let Err(e) = Self::start_input(
              &mut input_stream,
              &device_name,
              channel_count,
              sample_rate,
              buffer_size,
              rb_input_rb.clone(),
              engine.clone(),
            ) {
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
          AudioCommand::WriteOutput(samples) => {
            let (mut prod, _) = rb_output_rb.clone().split();
            for &sample in &samples {
              let _ = prod.try_push(sample);
            }
            drop(prod);
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
    device_name: &str,
    channel_count: u16,
    sample_rate: u32,
    buffer_size: usize,
    input_rb: Arc<HeapRb<f32>>,
    engine: Arc<Mutex<AudioEngine>>,
  ) -> Result<(), String> {
    let host = cpal::default_host();
    let device = Self::find_device(&host, device_name).ok_or_else(|| {
      format!("Device not found: {}", device_name)
    })?;

    let input_config = device
      .default_input_config()
      .map_err(|e| format!("Failed to get default config: {}", e))?;

    let stream_config = StreamConfig {
      channels: channel_count,
      sample_rate: cpal::SampleRate(sample_rate),
      buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
    };

    info!(
      "Starting input: {} ({} ch, {} Hz, {:?})",
      device_name,
      input_config.channels(),
      input_config.sample_rate().0,
      input_config.sample_format()
    );

    let input_rb = input_rb.clone();
    let engine = engine.clone();

    let new_stream = device
      .build_input_stream(
        &stream_config,
        move |data: &[f32], _: &_| {
          let (mut prod, _) = input_rb.clone().split();
          for &sample in data {
            let _ = prod.try_push(sample);
          }
          drop(prod);

          if let Ok(engine_lock) = engine.lock() {
            let _ = engine_lock.process_audio(data);
          }
        },
        move |err| {
          error!("Input stream error: {}", err);
        },
        None,
      )
      .map_err(|e| format!("Failed to build input stream: {}", e))?;

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
    let device = Self::find_device(&host, device_name).ok_or_else(|| {
      format!("Device not found: {}", device_name)
    })?;

    let output_config = device
      .default_output_config()
      .map_err(|e| format!("Failed to get default config: {}", e))?;

    let stream_config = StreamConfig {
      channels: channel_count,
      sample_rate: cpal::SampleRate(sample_rate),
      buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
    };

    info!(
      "Starting output: {} ({} ch, {} Hz, {:?})",
      device_name,
      output_config.channels(),
      output_config.sample_rate().0,
      output_config.sample_format()
    );

    let output_rb = output_rb.clone();

    let new_stream = device
      .build_output_stream(
        &stream_config,
        move |data: &mut [f32], _: &_| {
          let (_, mut cons) = output_rb.clone().split();
          for sample in data.iter_mut() {
            *sample = cons.try_pop().unwrap_or(0.0);
          }
          drop(cons);
        },
        move |err| {
          error!("Output stream error: {}", err);
        },
        None,
      )
      .map_err(|e| format!("Failed to build output stream: {}", e))?;

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
    host
      .devices()
      .ok()?
      .find(|d| {
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

  pub fn read_input(&self, buffer: &mut [f32]) -> usize {
    let (_, mut cons) = self.input_rb.clone().split();
    let mut count = 0;
    for sample in buffer.iter_mut() {
      *sample = cons.try_pop().unwrap_or(0.0);
      count += 1;
    }
    drop(cons);
    count
  }

  pub fn write_output(&self, samples: &[f32]) {
    let (mut prod, _) = self.output_rb.clone().split();
    for &sample in samples {
      let _ = prod.try_push(sample);
    }
    drop(prod);
  }
}
