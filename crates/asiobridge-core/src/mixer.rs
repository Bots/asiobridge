use std::sync::Arc;

/// Audio mixer — combines multiple input channels into a single output
#[derive(Debug, Clone)]
pub struct Mixer {
  pub channel_count: usize,
  pub sample_rate: u32,
  pub buffers: Arc<Vec<Vec<f32>>>,
}

impl Mixer {
  pub fn new(channel_count: usize, sample_rate: u32) -> Self {
    Self {
      channel_count,
      sample_rate,
      buffers: Arc::new(vec![vec![0.0f32; 2048]; channel_count]),
    }
  }

  pub fn mix(&self, inputs: &[&[f32]]) -> Vec<f32> {
    let mut output = vec![0.0f32; 2048];
    for input in inputs {
      for (i, &sample) in input.iter().enumerate() {
        output[i] += sample;
      }
    }
    // Normalize to prevent clipping
    let max = output.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
    if max > 1.0 {
      let gain = 0.99 / max;
      for sample in output.iter_mut() {
        *sample *= gain;
      }
    }
    output
  }
}
