/// Audio resampler — simple linear interpolation for now
/// TODO: Full rubato integration once Steinberg ASIO SDK is available
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Resampler {
    from_rate: u32,
    to_rate: u32,
    channels: usize,
}

impl Resampler {
    pub fn new(from_rate: u32, to_rate: u32, channels: usize) -> Self {
        Self {
            from_rate,
            to_rate,
            channels,
        }
    }

    pub fn resample(&self, input: &[f32]) -> Vec<f32> {
        if self.from_rate == self.to_rate || input.is_empty() {
            return input.to_vec();
        }

        let ratio = self.to_rate as f32 / self.from_rate as f32;
        let output_len = (input.len() as f32 * ratio) as usize;
        let mut output = vec![0.0f32; output_len];

        for (i, out_sample) in output.iter_mut().enumerate() {
            let src_pos = i as f32 / ratio;
            let idx = src_pos as usize;
            let frac = src_pos - idx as f32;

            if idx + 1 < input.len() {
                *out_sample = input[idx] * (1.0 - frac) + input[idx + 1] * frac;
            } else if idx < input.len() {
                *out_sample = input[idx];
            }
        }

        output
    }
}

impl Default for Resampler {
    fn default() -> Self {
        Self::new(44100, 44100, 2)
    }
}
