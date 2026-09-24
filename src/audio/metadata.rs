use std::fs::File;
use std::path::Path;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::{MetadataOptions, StandardTagKey, Tag, Value};
use symphonia::core::probe::Hint;

#[derive(Clone, Debug, Default)]
pub struct TrackMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub track_number: Option<u32>,
    pub duration_secs: f64,
    pub codec_name: String,
    pub sample_rate: Option<u32>,
    pub bits_per_sample: Option<u32>,
    pub channels: Option<u16>,
    pub bitrate_kbps: Option<u32>,
    pub album_art_bytes: Option<Vec<u8>>,
    pub album_art_mime: Option<String>,
}

impl TrackMetadata {
    /// Reads metadata, duration, format info, and album art from an audio file
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let p = path.as_ref();
        let file = File::open(p).map_err(|e| format!("Failed to open file: {}", e))?;
        let file_len = file.metadata().ok().map(|m| m.len());

        let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("Probe failed: {}", e))?;

        let mut format = probed.format;
        let mut meta = TrackMetadata::default();

        // 1. Check default track
        if let Some(track) = format.default_track() {
            let params = &track.codec_params;
            meta.sample_rate = params.sample_rate;
            meta.bits_per_sample = params.bits_per_sample;
            meta.channels = params.channels.map(|c| c.count() as u16);

            // Codec name
            meta.codec_name = match params.codec {
                symphonia::core::codecs::CODEC_TYPE_FLAC => "FLAC (Lossless)".to_string(),
                symphonia::core::codecs::CODEC_TYPE_MP3 => "MP3 (MPEG-1/2 Audio Layer III)".to_string(),
                symphonia::core::codecs::CODEC_TYPE_AAC => "AAC (Advanced Audio Coding)".to_string(),
                symphonia::core::codecs::CODEC_TYPE_VORBIS => "Ogg Vorbis".to_string(),
                symphonia::core::codecs::CODEC_TYPE_OPUS => "Opus Audio".to_string(),
                symphonia::core::codecs::CODEC_TYPE_ALAC => "ALAC (Apple Lossless)".to_string(),
                symphonia::core::codecs::CODEC_TYPE_PCM_S16LE
                | symphonia::core::codecs::CODEC_TYPE_PCM_S24LE
                | symphonia::core::codecs::CODEC_TYPE_PCM_S32LE
                | symphonia::core::codecs::CODEC_TYPE_PCM_F32LE => "WAV / PCM Linear".to_string(),
                c => format!("{:?}", c),
            };

            // Calculate duration
            if let Some(n_frames) = params.n_frames {
                if let Some(tb) = params.time_base {
                    let dur = tb.calc_time(n_frames);
                    meta.duration_secs = dur.seconds as f64 + dur.frac;
                } else if let Some(sr) = params.sample_rate {
                    meta.duration_secs = n_frames as f64 / sr as f64;
                }
            }

            // Estimate Bitrate if duration and file size are known
            if meta.duration_secs > 0.0 {
                if let Some(len) = file_len {
                    meta.bitrate_kbps = Some(((len * 8) as f64 / (meta.duration_secs * 1000.0)).round() as u32);
                }
            }
        }

        // 2. Extract metadata tags and album art from format metadata
        if let Some(metadata) = format.metadata().current() {
            Self::extract_from_tags(metadata.tags(), &mut meta);
            Self::extract_visuals(metadata.visuals(), &mut meta);
        }

        Ok(meta)
    }

    fn extract_from_tags(tags: &[Tag], meta: &mut TrackMetadata) {
        for tag in tags {
            if let Some(std_key) = tag.std_key {
                match std_key {
                    StandardTagKey::TrackTitle => {
                        meta.title = Some(tag.value.to_string());
                    }
                    StandardTagKey::Artist => {
                        meta.artist = Some(tag.value.to_string());
                    }
                    StandardTagKey::Album => {
                        meta.album = Some(tag.value.to_string());
                    }
                    StandardTagKey::Genre => {
                        meta.genre = Some(tag.value.to_string());
                    }
                    StandardTagKey::ReleaseDate | StandardTagKey::Date => {
                        if let Ok(y) = tag.value.to_string().trim().chars().take(4).collect::<String>().parse::<u32>() {
                            meta.year = Some(y);
                        }
                    }
                    StandardTagKey::TrackNumber => {
                        if let Value::UnsignedInt(n) = tag.value {
                            meta.track_number = Some(n as u32);
                        } else if let Ok(n) = tag.value.to_string().parse::<u32>() {
                            meta.track_number = Some(n);
                        }
                    }
                    _ => {}
                }
            } else {
                let key_lower = tag.key.to_lowercase();
                if key_lower == "title" && meta.title.is_none() {
                    meta.title = Some(tag.value.to_string());
                } else if (key_lower == "artist" || key_lower == "albumartist") && meta.artist.is_none() {
                    meta.artist = Some(tag.value.to_string());
                } else if key_lower == "album" && meta.album.is_none() {
                    meta.album = Some(tag.value.to_string());
                }
            }
        }
    }

    fn extract_visuals(visuals: &[symphonia::core::meta::Visual], meta: &mut TrackMetadata) {
        if meta.album_art_bytes.is_none() {
            if let Some(vis) = visuals.first() {
                meta.album_art_bytes = Some(vis.data.to_vec());
                meta.album_art_mime = Some(vis.media_type.clone());
            }
        }
    }
}
