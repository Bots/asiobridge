use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use ringbuf::{HeapRb, traits::*};
use std::sync::Arc;
use tracing::{error, info};

/// Audio stream manager using cpal for cross-platform audio I/O
pub struct AudioStream {
  stream: Option<Stream>,
  config: StreamConfig,
  is_running: bool,
  input_rb: Arc<HeapRb<f32>>,
  output_rb: Arc<HeapRb<f32>>,
}

impl AudioStream {
  pub fn new(channel_count: u16, sample_rate: u32, buffer_size: usize) -> Self {
    Self {
      stream: None,
      config: StreamConfig {
        channels: channel_count,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Fixed(buffer_size as u32),
      },
      is_running: false,
      input_rb: Arc::new(HeapRb::new(buffer_size * channel_count as usize)),
      output_rb: Arc::new(HeapRb::new(buffer_size * channel_count as usize)),
    }
  }

  pub fn get_input_devices(&self) -> Vec<String> {
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

  pub fn get_output_devices(&self) -> Vec<String> {
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

  pub fn get_default_input(&self) -> Option<String> {
    let host = cpal::default_host();
    host.default_input_device()
      .and_then(|d| d.name().ok())
      .map(|n| format!("Input: {}", n))
  }

  pub fn get_default_output(&self) -> Option<String> {
    let host = cpal::default_host();
    host.default_output_device()
      .and_then(|d| d.name().ok())
      .map(|n| format!("Output: {}", n))
  }

  pub fn start_input(
    &mut self,
    device_name: &str,
    on_data: impl Fn(&[f32]) + Send + 'static,
  ) -> Result<(), String> {
    let host = cpal::default_host();
    let device = self.find_device(&host, device_name).ok_or_else(|| {
      format!("Device not found: {}", device_name)
    })?;

    let config = device
      .default_input_config()
      .map_err(|e| format!("Failed to get default config: {}", e))?;

    info!(
      "Starting input: {} ({} ch, {} Hz, {:?})",
      device_name,
      config.channels(),
      config.sample_rate().0,
      config.sample_format()
    );

    let input_rb = self.input_rb.clone();
    let stream_config = self.config.clone();

    let stream = device
      .build_input_stream(
        &stream_config,
        move |data: &[f32], _: &_| {
          let (mut prod, _) = input_rb.clone().split();
          for &sample in data {
            let _ = prod.try_push(sample);
          }
          drop(prod);
          on_data(data);
        },
        move |err| {
          error!("Input stream error: {}", err);
        },
        None,
      )
      .map_err(|e| format!("Failed to build input stream: {}", e))?;

    stream.play().map_err(|e| format!("Failed to play: {}", e))?;

    self.stream = Some(stream);
    self.is_running = true;
    Ok(())
  }

  pub fn start_output(&mut self, device_name: &str) -> Result<(), String> {
    let host = cpal::default_host();
    let device = self.find_device(&host, device_name).ok_or_else(|| {
      format!("Device not found: {}", device_name)
    })?;

    let config = device
      .default_output_config()
      .map_err(|e| format!("Failed to get default config: {}", e))?;

    info!(
      "Starting output: {} ({} ch, {} Hz, {:?})",
      device_name,
      config.channels(),
      config.sample_rate().0,
      config.sample_format()
    );

    let output_rb = self.output_rb.clone();
    let stream_config = self.config.clone();

    let stream = device
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

    stream.play().map_err(|e| format!("Failed to play: {}", e))?;

    self.stream = Some(stream);
    self.is_running = true;
    Ok(())
  }

  pub fn stop(&mut self) {
    if let Some(stream) = self.stream.take() {
      let _ = stream.pause();
    }
    self.is_running = false;
  }

  pub fn is_running(&self) -> bool {
    self.is_running
  }

  pub fn write_output(&self, samples: &[f32]) {
    let (mut prod, _) = self.output_rb.clone().split();
    for &sample in samples {
      let _ = prod.try_push(sample);
    }
    drop(prod);
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

  pub fn get_sample_format(&self) -> Option<SampleFormat> {
    self.stream.as_ref().map(|_| SampleFormat::F32)
  }

  fn find_device(&self, host: &cpal::Host, name: &str) -> Option<cpal::Device> {
    host.devices()
      .ok()?
      .find(|d| d.name().ok().as_ref().map(|n| n == name || name.ends_with(n)).unwrap_or(false))
  }
}

impl Drop for AudioStream {
  fn drop(&mut self) {
    self.stop();
  }
}
