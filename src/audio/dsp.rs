use std::f32::consts::PI;

pub const EQ_FREQUENCIES: [f32; 10] = [
    31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0,
];

pub const EQ_LABELS: [&str; 10] = [
    "31Hz", "62Hz", "125Hz", "250Hz", "500Hz", "1kHz", "2kHz", "4kHz", "8kHz", "16kHz",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EqPreset {
    pub name: &'static str,
    pub bands: [f32; 10], // dB gains (-20.0 to +20.0)
    pub preamp: f32,      // dB
}

pub const EQ_PRESETS: &[EqPreset] = &[
    EqPreset {
        name: "Poweramp: Deep Bass & Clear Highs",
        bands: [6.5, 5.0, 3.2, 0.5, -0.5, 0.0, 1.8, 3.8, 5.5, 6.2],
        preamp: -1.5,
    },
    EqPreset {
        name: "Poweramp: Bass Boost",
        bands: [8.0, 6.5, 4.5, 2.0, 0.0, -0.5, 0.0, 1.0, 2.5, 3.0],
        preamp: -2.0,
    },
    EqPreset {
        name: "Poweramp: Vocal & Clarity",
        bands: [-1.5, -0.5, 0.0, 1.5, 3.2, 3.8, 3.0, 2.2, 1.5, 0.5],
        preamp: -0.5,
    },
    EqPreset {
        name: "Rock",
        bands: [5.0, 3.8, -1.0, -2.0, -0.5, 2.2, 3.8, 5.2, 6.0, 6.0],
        preamp: -1.0,
    },
    EqPreset {
        name: "Pop / Modern Beats",
        bands: [4.0, 3.0, 1.5, 2.0, 2.5, -0.5, -1.0, 1.2, 3.5, 4.0],
        preamp: -1.0,
    },
    EqPreset {
        name: "Hip-Hop / Heavy Bass",
        bands: [7.5, 6.0, 3.5, 1.0, -1.0, -0.5, 1.5, 2.8, 4.0, 4.5],
        preamp: -2.0,
    },
    EqPreset {
        name: "EDM / Electronic",
        bands: [6.5, 5.0, 1.5, -1.5, -1.0, 1.0, 3.5, 5.0, 5.5, 5.0],
        preamp: -1.5,
    },
    EqPreset {
        name: "Jazz & Acoustic",
        bands: [3.5, 2.5, 1.5, 1.8, -1.0, -1.0, 0.5, 1.8, 3.0, 3.8],
        preamp: -0.5,
    },
    EqPreset {
        name: "Classical / Symphony",
        bands: [4.0, 3.0, 2.0, 1.5, -1.0, -1.0, 0.5, 2.2, 3.2, 4.0],
        preamp: -0.5,
    },
    EqPreset {
        name: "Headphones Audiophile",
        bands: [4.5, 3.0, 1.0, -1.0, -1.2, 0.5, 2.0, 3.5, 5.0, 6.0],
        preamp: -1.0,
    },
    EqPreset {
        name: "Flat (Studio Reference)",
        bands: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        preamp: 0.0,
    },
];

/// Second-order Biquad IIR filter coefficients for peaking EQ
#[derive(Clone, Copy, Debug)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State memory for Stereo channels: [Channel 0, Channel 1]
    x1: [f32; 2],
    x2: [f32; 2],
    y1: [f32; 2],
    y2: [f32; 2],
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: [0.0, 0.0],
            x2: [0.0, 0.0],
            y1: [0.0, 0.0],
            y2: [0.0, 0.0],
        }
    }
}

impl BiquadFilter {
    pub fn update_peaking(&mut self, sample_rate: f32, center_freq: f32, gain_db: f32, q: f32) {
        if gain_db.abs() < 0.01 {
            // Flat pass-through
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        let w0 = 2.0 * PI * (center_freq / sample_rate).clamp(0.0001, 0.499);
        let alpha = w0.sin() / (2.0 * q.max(0.1));
        let a = 10.0f32.powf(gain_db / 40.0);

        let cos_w0 = w0.cos();
        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    pub fn update_low_shelf(&mut self, sample_rate: f32, cutoff_freq: f32, gain_db: f32, s: f32) {
        if gain_db.abs() < 0.01 {
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (cutoff_freq / sample_rate).clamp(0.0001, 0.499);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = (sin_w0 / 2.0) * ((a + 1.0 / a) * (1.0 / s - 1.0) + 2.0).max(0.0).sqrt();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    pub fn update_high_shelf(&mut self, sample_rate: f32, cutoff_freq: f32, gain_db: f32, s: f32) {
        if gain_db.abs() < 0.01 {
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;
            self.a1 = 0.0;
            self.a2 = 0.0;
            return;
        }

        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (cutoff_freq / sample_rate).clamp(0.0001, 0.499);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = (sin_w0 / 2.0) * ((a + 1.0 / a) * (1.0 / s - 1.0) + 2.0).max(0.0).sqrt();
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - two_sqrt_a_alpha;

        let inv_a0 = 1.0 / a0;
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    #[inline]
    pub fn process_sample(&mut self, ch: usize, input: f32) -> f32 {
        let ch = ch.min(1);
        let output = self.b0 * input + self.b1 * self.x1[ch] + self.b2 * self.x2[ch]
            - self.a1 * self.y1[ch]
            - self.a2 * self.y2[ch];

        self.x2[ch] = self.x1[ch];
        self.x1[ch] = input;
        self.y2[ch] = self.y1[ch];
        self.y1[ch] = output;

        output
    }

    pub fn reset(&mut self) {
        self.x1 = [0.0, 0.0];
        self.x2 = [0.0, 0.0];
        self.y1 = [0.0, 0.0];
        self.y2 = [0.0, 0.0];
    }
}

/// 10-band Graphic Equalizer processor
#[derive(Clone, Debug)]
pub struct Equalizer {
    pub enabled: bool,
    pub preamp_db: f32,
    pub bands_db: [f32; 10],
    filters: [BiquadFilter; 10],
    sample_rate: f32,
}

impl Default for Equalizer {
    fn default() -> Self {
        let mut eq = Self {
            enabled: true,
            preamp_db: 0.0,
            bands_db: [0.0; 10],
            filters: [BiquadFilter::default(); 10],
            sample_rate: 44100.0,
        };
        eq.recalculate();
        eq
    }
}

impl Equalizer {
    pub fn new(sample_rate: f32) -> Self {
        let mut eq = Self {
            enabled: true,
            preamp_db: 0.0,
            bands_db: [0.0; 10],
            filters: [BiquadFilter::default(); 10],
            sample_rate: sample_rate.max(8000.0),
        };
        eq.recalculate();
        eq
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        if (self.sample_rate - sample_rate).abs() > 1.0 {
            self.sample_rate = sample_rate.max(8000.0);
            self.recalculate();
        }
    }

    pub fn set_band(&mut self, band_idx: usize, gain_db: f32) {
        if band_idx < 10 {
            let clamped = gain_db.clamp(-20.0, 20.0);
            self.bands_db[band_idx] = clamped;
            self.update_filter_band(band_idx);
        }
    }

    fn update_filter_band(&mut self, i: usize) {
        let gain = self.bands_db[i];
        let freq = EQ_FREQUENCIES[i];

        if i == 0 {
            // Band 0 (31 Hz): Low Shelf for Poweramp deep sub-bass response
            self.filters[i].update_low_shelf(self.sample_rate, 45.0, gain, 0.9);
        } else if i == 9 {
            // Band 9 (16 kHz): High Shelf for silky airy highs
            self.filters[i].update_high_shelf(self.sample_rate, 14000.0, gain, 0.9);
        } else {
            // Bands 1-8: Peaking with tuned Q factors for optimal musical overlap
            let q = match i {
                1 => 1.2, // 62Hz bass punch
                2 => 1.3, // 125Hz warmth
                3 => 1.4, // 250Hz low-mid body
                4 => 1.4, // 500Hz mid
                5 => 1.4, // 1kHz vocal presence
                6 => 1.3, // 2kHz upper mid
                7 => 1.2, // 4kHz presence
                8 => 1.1, // 8kHz brilliance
                _ => 1.414,
            };
            self.filters[i].update_peaking(self.sample_rate, freq, gain, q);
        }
    }

    pub fn set_preamp(&mut self, preamp_db: f32) {
        self.preamp_db = preamp_db.clamp(-20.0, 20.0);
    }

    pub fn apply_preset(&mut self, preset: &EqPreset) {
        self.preamp_db = preset.preamp;
        self.bands_db = preset.bands;
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        for i in 0..10 {
            self.update_filter_band(i);
        }
    }

    pub fn reset_state(&mut self) {
        for f in &mut self.filters {
            f.reset();
        }
    }

    #[inline]
    pub fn process_sample(&mut self, ch: usize, mut sample: f32) -> f32 {
        if !self.enabled {
            return sample;
        }

        // Apply pre-amp
        let preamp_gain = 10.0f32.powf(self.preamp_db / 20.0);
        sample *= preamp_gain;

        // Cascade 10 precision filters
        for filter in &mut self.filters {
            sample = filter.process_sample(ch, sample);
        }

        sample
    }
}

/// Volume, Panning, and Studio Soft-Clipping Limiter (Poweramp-style warm analog limiter)
#[derive(Clone, Debug)]
pub struct MasterDsp {
    pub volume: f32, // 0.0 to 2.0 (0% to 200%)
    pub pan: f32,    // -1.0 (Left) to +1.0 (Right)
    pub muted: bool,
    pub speed: f32,  // 0.25 to 8.0
}

impl Default for MasterDsp {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pan: 0.0,
            muted: false,
            speed: 1.0,
        }
    }
}

impl MasterDsp {
    #[inline]
    pub fn process_stereo(&self, left: f32, right: f32) -> (f32, f32) {
        if self.muted {
            return (0.0, 0.0);
        }

        let vol = self.volume.clamp(0.0, 2.0);

        // Constant-power pan rule
        let pan_clamped = self.pan.clamp(-1.0, 1.0);
        let left_gain = if pan_clamped > 0.0 {
            1.0 - pan_clamped
        } else {
            1.0
        };
        let right_gain = if pan_clamped < 0.0 {
            1.0 + pan_clamped
        } else {
            1.0
        };

        let mut out_l = left * vol * left_gain;
        let mut out_r = right * vol * right_gain;

        // Studio warm analog soft-knee saturation: preserves dynamic punch without harsh clipping
        #[inline]
        fn soft_clip(x: f32) -> f32 {
            if x > 0.95 {
                0.95 + 0.05 * ((x - 0.95) / 0.05).tanh()
            } else if x < -0.95 {
                -0.95 + 0.05 * ((x + 0.95) / 0.05).tanh()
            } else {
                x
            }
        }

        out_l = soft_clip(out_l);
        out_r = soft_clip(out_r);

        (out_l, out_r)
    }
}
