use crate::audio::decoder::AudioDecoder;
use crate::audio::dsp::{Equalizer, MasterDsp};
use crate::audio::visualizer::VisualizerBuffer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use crossbeam_channel::{Receiver, Sender};
use rtrb::RingBuffer;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

pub enum AudioCommand {
    Play(String, bool), // (path_or_url, is_stream)
    Pause,
    Resume,
    Stop,
    Seek(f64),
    SetVolume(f32),
    SetPan(f32),
    SetSpeed(f32),
    SetMute(bool),
    SetEqEnabled(bool),
    SetEqPreamp(f32),
    SetEqBand(usize, f32),
    SetEqBands([f32; 10]),
    SetAbLoop(Option<(f64, f64)>),
}

pub struct PlaybackStatus {
    pub state: PlaybackState,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub sample_rate: u32,
    pub channels: u16,
    pub is_seekable: bool,
    pub error: Option<String>,
    pub track_finished: bool,
}

pub struct AudioBackend {
    cmd_tx: Sender<AudioCommand>,
    status_rx: Receiver<PlaybackStatus>,
    _stream: Stream,
    pub visualizer_buf: VisualizerBuffer,
}

impl AudioBackend {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device found on system".to_string())?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| format!("Default output config error: {}", e))?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels() as usize;

        // Create ring buffer for 0.25 seconds of stereo audio frames
        let ring_capacity = (sample_rate as usize * 2).next_power_of_two();
        let (producer, mut consumer) = RingBuffer::<f32>::new(ring_capacity);

        let visualizer_buf = VisualizerBuffer::new();
        let vis_buf_clone = visualizer_buf.clone();

        // CPAL output callback
        let stream = match supported_config.sample_format() {
            SampleFormat::F32 => {
                let vis_buf = vis_buf_clone.clone();
                device
                    .build_output_stream(
                        &supported_config.into(),
                        move |data: &mut [f32], _| {
                            let mut temp_vis = Vec::with_capacity(data.len() / channels);
                            for frame in data.chunks_mut(channels) {
                                let l = consumer.pop().unwrap_or(0.0);
                                let r = consumer.pop().unwrap_or(l);

                                if frame.len() >= 2 {
                                    frame[0] = l;
                                    frame[1] = r;
                                } else if !frame.is_empty() {
                                    frame[0] = (l + r) * 0.5;
                                }

                                temp_vis.push((l + r) * 0.5);
                            }
                            vis_buf.push_samples(&temp_vis);
                        },
                        |err| eprintln!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            _ => return Err("Unsupported audio hardware format (requires f32)".to_string()),
        };

        stream.play().map_err(|e| format!("Failed to start audio stream: {}", e))?;

        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded::<AudioCommand>();
        let (status_tx, status_rx) = crossbeam_channel::unbounded::<PlaybackStatus>();

        // Spawn audio decode and DSP worker thread
        let mut worker = AudioWorker {
            cmd_rx,
            status_tx,
            ring_producer: producer,
            device_sample_rate: sample_rate,
            decoder: None,
            equalizer: Equalizer::new(sample_rate as f32),
            dsp: MasterDsp::default(),
            state: PlaybackState::Stopped,
            position_secs: 0.0,
            duration_secs: 0.0,
            ab_loop: None,
            // Resampling state
            phase: 0.0,
            curr_l: 0.0,
            curr_r: 0.0,
            next_l: 0.0,
            next_r: 0.0,
        };

        thread::Builder::new()
            .name("xorin-audio-worker".to_string())
            .spawn(move || {
                worker.run();
            })
            .map_err(|e| format!("Failed to spawn audio worker thread: {}", e))?;

        Ok(Self {
            cmd_tx,
            status_rx,
            _stream: stream,
            visualizer_buf,
        })
    }

    pub fn send_command(&self, cmd: AudioCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    pub fn try_recv_status(&self) -> Option<PlaybackStatus> {
        let mut latest = None;
        while let Ok(status) = self.status_rx.try_recv() {
            latest = Some(status);
        }
        latest
    }
}

struct AudioWorker {
    cmd_rx: Receiver<AudioCommand>,
    status_tx: Sender<PlaybackStatus>,
    ring_producer: rtrb::Producer<f32>,
    device_sample_rate: u32,
    decoder: Option<AudioDecoder>,
    equalizer: Equalizer,
    dsp: MasterDsp,
    state: PlaybackState,
    position_secs: f64,
    duration_secs: f64,
    ab_loop: Option<(f64, f64)>,
    phase: f64,
    curr_l: f32,
    curr_r: f32,
    next_l: f32,
    next_r: f32,
}

impl AudioWorker {
    fn run(&mut self) {
        let mut last_status_tick = std::time::Instant::now();

        loop {
            // Process commands
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                self.handle_command(cmd);
            }

            if self.state == PlaybackState::Playing {
                self.process_audio_chunk();
            } else {
                // Idle sleep when stopped or paused
                thread::sleep(Duration::from_millis(10));
            }

            // Send periodic status updates (~30 times a second)
            if last_status_tick.elapsed() >= Duration::from_millis(33) {
                last_status_tick = std::time::Instant::now();
                self.send_status(false, None);
            }
        }
    }

    fn handle_command(&mut self, cmd: AudioCommand) {
        match cmd {
            AudioCommand::Play(path_or_url, is_stream) => {
                self.state = PlaybackState::Buffering;
                self.send_status(false, None);

                let res = if is_stream {
                    AudioDecoder::open_stream(&path_or_url)
                } else {
                    AudioDecoder::open_file(&path_or_url)
                };

                match res {
                    Ok(mut dec) => {
                        self.position_secs = 0.0;
                        self.equalizer.set_sample_rate(dec.sample_rate as f32);
                        self.equalizer.reset_state();

                        // Fetch initial pair
                        if let Ok(Some((l, r))) = dec.next_stereo_sample() {
                            self.curr_l = l;
                            self.curr_r = r;
                        }
                        if let Ok(Some((l, r))) = dec.next_stereo_sample() {
                            self.next_l = l;
                            self.next_r = r;
                        }

                        self.phase = 0.0;
                        self.decoder = Some(dec);
                        self.state = PlaybackState::Playing;
                        self.send_status(false, None);
                    }
                    Err(e) => {
                        self.state = PlaybackState::Stopped;
                        self.decoder = None;
                        self.send_status(false, Some(e));
                    }
                }
            }
            AudioCommand::Pause => {
                if self.state == PlaybackState::Playing {
                    self.state = PlaybackState::Paused;
                    self.send_status(false, None);
                }
            }
            AudioCommand::Resume => {
                if self.state == PlaybackState::Paused {
                    self.state = PlaybackState::Playing;
                    self.send_status(false, None);
                }
            }
            AudioCommand::Stop => {
                self.state = PlaybackState::Stopped;
                self.decoder = None;
                self.position_secs = 0.0;
                self.send_status(false, None);
            }
            AudioCommand::Seek(secs) => {
                if let Some(ref mut dec) = self.decoder {
                    if let Ok(actual) = dec.seek_to(secs) {
                        self.position_secs = actual;
                        self.equalizer.reset_state();
                        self.phase = 0.0;
                        if let Ok(Some((l, r))) = dec.next_stereo_sample() {
                            self.curr_l = l;
                            self.curr_r = r;
                        }
                        if let Ok(Some((l, r))) = dec.next_stereo_sample() {
                            self.next_l = l;
                            self.next_r = r;
                        }
                        self.send_status(false, None);
                    }
                }
            }
            AudioCommand::SetVolume(v) => self.dsp.volume = v,
            AudioCommand::SetPan(p) => self.dsp.pan = p,
            AudioCommand::SetSpeed(s) => self.dsp.speed = s.clamp(0.25, 8.0),
            AudioCommand::SetMute(m) => self.dsp.muted = m,
            AudioCommand::SetEqEnabled(en) => self.equalizer.enabled = en,
            AudioCommand::SetEqPreamp(preamp) => self.equalizer.set_preamp(preamp),
            AudioCommand::SetEqBand(idx, gain) => self.equalizer.set_band(idx, gain),
            AudioCommand::SetEqBands(bands) => {
                for (i, &g) in bands.iter().enumerate() {
                    self.equalizer.set_band(i, g);
                }
            }
            AudioCommand::SetAbLoop(ab) => self.ab_loop = ab,
        }
    }

    fn process_audio_chunk(&mut self) {
        let dec = match self.decoder.as_mut() {
            Some(d) => d,
            None => {
                self.state = PlaybackState::Stopped;
                return;
            }
        };

        // Check A-B loop point
        if let Some((a, b)) = self.ab_loop {
            if b > a && dec.current_time_secs >= b {
                let _ = dec.seek_to(a);
                self.position_secs = a;
            }
        }

        // We fill ring buffer with stereo frames
        let slots = self.ring_producer.slots();
        if slots < 64 {
            thread::sleep(Duration::from_millis(2));
            return;
        }

        let frames_to_generate = (slots / 2).min(512);
        let dec_rate = dec.sample_rate as f64;
        let out_rate = self.device_sample_rate as f64;
        let speed = self.dsp.speed as f64;
        let step = (dec_rate * speed) / out_rate;

        let mut hit_eof = false;

        for _ in 0..frames_to_generate {
            // Linear interpolation between curr and next
            let frac = self.phase as f32;
            let raw_l = self.curr_l + (self.next_l - self.curr_l) * frac;
            let raw_r = self.curr_r + (self.next_r - self.curr_r) * frac;

            // 1. 10-Band Graphic Equalizer & Preamp
            let eq_l = self.equalizer.process_sample(0, raw_l);
            let eq_r = self.equalizer.process_sample(1, raw_r);

            // 2. Master DSP (Volume boost up to 200%, Pan, Soft-Limiter)
            let (out_l, out_r) = self.dsp.process_stereo(eq_l, eq_r);

            let _ = self.ring_producer.push(out_l);
            let _ = self.ring_producer.push(out_r);

            self.phase += step;
            while self.phase >= 1.0 {
                self.phase -= 1.0;
                self.curr_l = self.next_l;
                self.curr_r = self.next_r;

                match dec.next_stereo_sample() {
                    Ok(Some((nl, nr))) => {
                        self.next_l = nl;
                        self.next_r = nr;
                    }
                    Ok(None) => {
                        hit_eof = true;
                        break;
                    }
                    Err(_) => {
                        hit_eof = true;
                        break;
                    }
                }
            }

            if hit_eof {
                break;
            }
        }

        self.position_secs = dec.current_time_secs;

        if hit_eof {
            self.state = PlaybackState::Stopped;
            self.decoder = None;
            self.send_status(true, None);
        }
    }

    fn send_status(&self, track_finished: bool, error: Option<String>) {
        let (sr, ch, is_seekable) = match self.decoder.as_ref() {
            Some(d) => (d.sample_rate, d.channels, d.is_seekable),
            None => (self.device_sample_rate, 2, true),
        };

        let status = PlaybackStatus {
            state: self.state,
            position_secs: self.position_secs,
            duration_secs: self.duration_secs,
            sample_rate: sr,
            channels: ch,
            is_seekable,
            error,
            track_finished,
        };
        let _ = self.status_tx.send(status);
    }
}
