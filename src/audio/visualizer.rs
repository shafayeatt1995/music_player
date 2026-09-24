use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const FFT_SIZE: usize = 1024;
pub const NUM_SPECTRUM_BARS: usize = 48;

#[derive(Clone)]
pub struct VisualizerBuffer {
    buffer: Arc<Mutex<Vec<f32>>>,
    pub is_active: Arc<AtomicBool>,
}

impl Default for VisualizerBuffer {
    fn default() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(vec![0.0; FFT_SIZE])),
            is_active: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl VisualizerBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Called from audio playback callback thread to push fresh mono/downmixed samples
    pub fn push_samples(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        if let Ok(mut buf) = self.buffer.try_lock() {
            let n = samples.len();
            if n >= FFT_SIZE {
                buf.copy_from_slice(&samples[n - FFT_SIZE..]);
            } else {
                buf.drain(0..n);
                buf.extend_from_slice(samples);
            }
            self.is_active.store(true, Ordering::Relaxed);
        }
    }

    /// Read raw samples snapshot for rendering waveform / oscilloscope
    pub fn get_samples(&self) -> Vec<f32> {
        if let Ok(buf) = self.buffer.lock() {
            buf.clone()
        } else {
            vec![0.0; FFT_SIZE]
        }
    }
}

pub struct VisualizerProcessor {
    planner: FftPlanner<f32>,
    hann_window: [f32; FFT_SIZE],
    fft_buffer: Vec<Complex<f32>>,
    current_bars: [f32; NUM_SPECTRUM_BARS],
    peak_bars: [f32; NUM_SPECTRUM_BARS],
}

impl Default for VisualizerProcessor {
    fn default() -> Self {
        let mut hann = [0.0f32; FFT_SIZE];
        for i in 0..FFT_SIZE {
            hann[i] = 0.5 * (1.0 - (2.0 * PI * i as f32 / (FFT_SIZE as f32 - 1.0)).cos());
        }

        Self {
            planner: FftPlanner::new(),
            hann_window: hann,
            fft_buffer: vec![Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            current_bars: [0.0; NUM_SPECTRUM_BARS],
            peak_bars: [0.0; NUM_SPECTRUM_BARS],
        }
    }
}

impl VisualizerProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes FFT and updates smoothed spectrum bars and peak caps
    pub fn process_spectrum(&mut self, time_samples: &[f32], decay_factor: f32) -> ([f32; NUM_SPECTRUM_BARS], [f32; NUM_SPECTRUM_BARS]) {
        let n = time_samples.len().min(FFT_SIZE);

        // Windowing
        for i in 0..FFT_SIZE {
            let sample = if i < n { time_samples[i] } else { 0.0 };
            self.fft_buffer[i] = Complex {
                re: sample * self.hann_window[i],
                im: 0.0,
            };
        }

        let fft = self.planner.plan_fft_forward(FFT_SIZE);
        fft.process(&mut self.fft_buffer);

        // Frequency binning to logarithmic bars
        let half_size = FFT_SIZE / 2;
        let mut raw_bars = [0.0f32; NUM_SPECTRUM_BARS];

        for i in 0..NUM_SPECTRUM_BARS {
            // Logarithmic mapping: lower bars span fewer bins, higher bars span wider bins
            let low_frac = (i as f32 / NUM_SPECTRUM_BARS as f32).powf(2.2);
            let high_frac = ((i + 1) as f32 / NUM_SPECTRUM_BARS as f32).powf(2.2);

            let low_bin = (low_frac * (half_size - 1) as f32).floor() as usize;
            let high_bin = ((high_frac * (half_size - 1) as f32).ceil() as usize).max(low_bin + 1).min(half_size);

            let mut sum_mag = 0.0f32;
            let count = (high_bin - low_bin) as f32;
            for bin in low_bin..high_bin {
                let c = self.fft_buffer[bin];
                let mag = (c.re * c.re + c.im * c.im).sqrt();
                sum_mag += mag;
            }

            let avg_mag = (sum_mag / count) / (FFT_SIZE as f32 * 0.25);
            // Apply slight tilt boost to high frequencies so spectrum looks balanced across bands
            let tilt = 1.0 + (i as f32 / NUM_SPECTRUM_BARS as f32) * 2.0;
            let normalized = (avg_mag * tilt * 1.8).clamp(0.0, 1.0);
            raw_bars[i] = normalized;
        }

        // Apply smoothing and gravity falloff
        let _smoothing = 0.65;
        for i in 0..NUM_SPECTRUM_BARS {
            let target = raw_bars[i];
            if target > self.current_bars[i] {
                self.current_bars[i] = target;
            } else {
                self.current_bars[i] = self.current_bars[i] * decay_factor;
            }

            // Peaks
            if self.current_bars[i] > self.peak_bars[i] {
                self.peak_bars[i] = self.current_bars[i];
            } else {
                self.peak_bars[i] = (self.peak_bars[i] - 0.008).max(0.0);
            }
        }

        (self.current_bars, self.peak_bars)
    }
}
