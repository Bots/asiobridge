use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::PathBuf;
use tracing::{error, info};

/// Audio recorder that writes to WAV files
pub struct AudioRecorder {
    output_dir: PathBuf,
    is_recording: bool,
    sample_rate: u32,
    bit_depth: u16,
    channels: u16,
    writer: Option<BufWriter<File>>,
    samples_written: u32,
}

impl AudioRecorder {
    pub fn new(output_dir: PathBuf, sample_rate: u32, bit_depth: u16, channels: u16) -> Self {
        std::fs::create_dir_all(&output_dir).ok();
        Self {
            output_dir,
            is_recording: false,
            sample_rate,
            bit_depth,
            channels,
            writer: None,
            samples_written: 0,
        }
    }

    pub fn start_recording(&mut self) -> bool {
        if self.is_recording {
            return false;
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!(
            "recording_{}ch_{}hz_{}.wav",
            self.channels, self.sample_rate, timestamp
        );
        let filepath = self.output_dir.join(filename);

        match File::create(&filepath) {
            Ok(file) => {
                let writer = BufWriter::new(file);
                info!("Starting recording to {}", filepath.display());
                self.writer = Some(writer);
                self.is_recording = true;
                self.samples_written = 0;
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

        let num_frames = self.samples_written / (self.channels as u32);
        let bytes_per_sample = (self.bit_depth as u32) / 8;
        let data_size = (num_frames as u64) * (self.channels as u64) * (bytes_per_sample as u64);
        let file_size = 36 + data_size;

        if let Some(mut writer) = self.writer.take() {
            // Write WAV header at the beginning of the file
            let _ = writer.seek(std::io::SeekFrom::Start(0));

            let mut buf = Vec::with_capacity(44);
            buf.extend_from_slice(b"RIFF");
            buf.extend_from_slice(&(file_size as u32).to_le_bytes());
            buf.extend_from_slice(b"WAVE");
            buf.extend_from_slice(b"fmt ");
            buf.extend_from_slice(&16u32.to_le_bytes());
            buf.extend_from_slice(&1u16.to_le_bytes());
            buf.extend_from_slice(&self.channels.to_le_bytes());
            buf.extend_from_slice(&self.sample_rate.to_le_bytes());
            buf.extend_from_slice(
                &(self.sample_rate * bytes_per_sample * self.channels as u32).to_le_bytes(),
            );
            buf.extend_from_slice(&((bytes_per_sample as u16) * self.channels).to_le_bytes());
            buf.extend_from_slice(&self.bit_depth.to_le_bytes());
            buf.extend_from_slice(b"data");
            buf.extend_from_slice(&(data_size as u32).to_le_bytes());

            let _ = writer.write_all(&buf);
            let _ = writer.flush();
        }

        info!(
            "Stopping recording: {} frames, {} ch, {}Hz, {}bit",
            num_frames, self.channels, self.sample_rate, self.bit_depth
        );

        self.is_recording = false;
        true
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn write_samples(&mut self, samples: &[f32]) {
        if !self.is_recording || self.writer.is_none() {
            return;
        }

        let i16_samples: Vec<i16> = samples
            .iter()
            .map(|&s| {
                let clamped = s.clamp(-1.0, 1.0);
                (clamped * 32767.0) as i16
            })
            .collect();

        if let Some(ref mut writer) = self.writer {
            if let Err(e) = writer.write_all(bytemuck::cast_slice(&i16_samples)) {
                error!("Failed to write audio samples: {}", e);
                self.is_recording = false;
                self.writer = None;
                return;
            }
            self.samples_written += i16_samples.len() as u32;
        }
    }

    pub fn get_output_dir(&self) -> PathBuf {
        self.output_dir.clone()
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_channels(&mut self, channels: u16) {
        self.channels = channels;
    }
}
