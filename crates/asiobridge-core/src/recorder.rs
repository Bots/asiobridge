use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// Audio recorder that writes to FLAC files
pub struct AudioRecorder {
    output_dir: PathBuf,
    is_recording: bool,
    sample_rate: u32,
    bit_depth: u32,
    channels: u16,
    buffer: Arc<Mutex<Vec<f32>>>,
    file_handle: Option<File>,
}

impl AudioRecorder {
    pub fn new(output_dir: PathBuf, sample_rate: u32, bit_depth: u32, channels: u16) -> Self {
        std::fs::create_dir_all(&output_dir).ok();
        Self {
            output_dir,
            is_recording: false,
            sample_rate,
            bit_depth,
            channels,
            buffer: Arc::new(Mutex::new(Vec::new())),
            file_handle: None,
        }
    }

    pub fn start_recording(&mut self) -> bool {
        if self.is_recording {
            return false;
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "recording_{}ch_{}hz_{}.flac",
            self.channels, self.sample_rate, timestamp
        );
        let filepath = self.output_dir.join(filename);

        match File::create(&filepath) {
            Ok(file) => {
                info!("Starting recording to {}", filepath.display());
                self.file_handle = Some(file);
                self.is_recording = true;
                true
            }
            Err(e) => {
                error!("Failed to create recording file: {}", e);
                false
            }
        }
    }

    pub fn stop_recording(&mut self) -> bool {
        if !self.is_recording {
            return false;
        }

        info!("Stopping recording");
        self.is_recording = false;
        self.file_handle = None;
        true
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn write_samples(&mut self, samples: &[f32]) {
        if !self.is_recording {
            return;
        }

        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.extend_from_slice(samples);
        }
    }

    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }
}
