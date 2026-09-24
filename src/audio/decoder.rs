use std::fs::File;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

pub struct AudioDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    pub sample_rate: u32,
    pub channels: u16,
    time_base: Option<TimeBase>,
    pub current_time_secs: f64,
    pub is_seekable: bool,
    sample_buf: Option<SampleBuffer<f32>>,
    sample_index: usize,
}

impl AudioDecoder {
    pub fn open_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let file = File::open(p).map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        Self::init(mss, hint, true)
    }

    pub fn open_stream(url: &str) -> Result<Self, String> {
        let resp = ureq::get(url)
            .call()
            .map_err(|e| format!("Stream connection failed: {}", e))?;

        let reader = resp.into_reader();
        let read_only = ReadOnlySource::new(reader);
        let mss = MediaSourceStream::new(Box::new(read_only), MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        if url.ends_with(".mp3") {
            hint.with_extension("mp3");
        } else if url.ends_with(".ogg") || url.ends_with(".opus") {
            hint.with_extension("ogg");
        } else if url.ends_with(".aac") {
            hint.with_extension("aac");
        } else if url.ends_with(".flac") {
            hint.with_extension("flac");
        }

        Self::init(mss, hint, false)
    }

    fn init(mss: MediaSourceStream, hint: Hint, is_seekable: bool) -> Result<Self, String> {
        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("Format probe failed: {}", e))?;

        let format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| "No default audio track found".to_string())?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
        let time_base = track.codec_params.time_base;

        let dec_opts = DecoderOptions::default();
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| format!("Decoder creation failed: {}", e))?;

        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            channels,
            time_base,
            current_time_secs: 0.0,
            is_seekable,
            sample_buf: None,
            sample_index: 0,
        })
    }

    pub fn seek_to(&mut self, time_secs: f64) -> Result<f64, String> {
        if !self.is_seekable {
            return Err("Stream is not seekable".to_string());
        }

        let seek_time = Time::from(time_secs);
        let seek_res = self
            .format
            .seek(
                SeekMode::Coarse,
                SeekTo::Time {
                    time: seek_time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| format!("Seek failed: {}", e))?;

        self.decoder.reset();
        self.sample_buf = None;
        self.sample_index = 0;

        let actual_secs = if let Some(tb) = self.time_base {
            let actual_time = tb.calc_time(seek_res.actual_ts);
            actual_time.seconds as f64 + actual_time.frac
        } else {
            time_secs
        };

        self.current_time_secs = actual_secs;
        Ok(actual_secs)
    }

    /// Read next interleaved stereo [L, R] sample pair. Returns None on EOF.
    pub fn next_stereo_sample(&mut self) -> Result<Option<(f32, f32)>, String> {
        loop {
            // If we have samples in current sample_buf, return next stereo pair
            if let Some(ref buf) = self.sample_buf {
                let samples = buf.samples();
                let num_ch = self.channels as usize;

                if self.sample_index + num_ch <= samples.len() {
                    let (l, r) = if num_ch == 1 {
                        let s = samples[self.sample_index];
                        self.sample_index += 1;
                        (s, s)
                    } else {
                        let l = samples[self.sample_index];
                        let r = samples[self.sample_index + 1];
                        self.sample_index += num_ch;
                        (l, r)
                    };

                    self.current_time_secs += 1.0 / self.sample_rate as f64;
                    return Ok(Some((l, r)));
                } else {
                    // Exhausted current buffer, decode next packet
                    self.sample_buf = None;
                    self.sample_index = 0;
                }
            }

            // Fetch next packet
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(None);
                }
                Err(SymphoniaError::IoError(_)) => return Ok(None),
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(e) => return Err(format!("Decode packet error: {}", e)),
            };

            // Check if packet belongs to our audio track
            if packet.track_id() != self.track_id {
                continue;
            }

            // Update timestamp
            if let Some(tb) = self.time_base {
                let packet_time = tb.calc_time(packet.ts());
                self.current_time_secs = packet_time.seconds as f64 + packet_time.frac;
            }

            // Decode packet
            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut sbuf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                    sbuf.copy_interleaved_ref(decoded);
                    self.sample_buf = Some(sbuf);
                    self.sample_index = 0;
                }
                Err(SymphoniaError::DecodeError(_)) => {
                    // Corrupted frame, skip to next packet
                    continue;
                }
                Err(e) => return Err(format!("Decoder error: {}", e)),
            }
        }
    }
}
