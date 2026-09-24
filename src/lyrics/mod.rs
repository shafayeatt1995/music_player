use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct LyricLine {
    pub time_secs: f64,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct Lyrics {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub offset_ms: f64,
    pub lines: Vec<LyricLine>,
    pub is_synced: bool,
}

impl Lyrics {
    /// Attempts to load lyrics from an LRC file matching the audio file path or plain text lyrics.
    pub fn load_for_audio_file<P: AsRef<Path>>(audio_path: P) -> Option<Self> {
        let p = audio_path.as_ref();
        let lrc_path = p.with_extension("lrc");
        if lrc_path.exists() {
            if let Ok(content) = fs::read_to_string(&lrc_path) {
                return Some(Self::parse(&content));
            }
        }

        // Try with uppercase extension .LRC
        let lrc_upper = p.with_extension("LRC");
        if lrc_upper.exists() {
            if let Ok(content) = fs::read_to_string(&lrc_upper) {
                return Some(Self::parse(&content));
            }
        }

        // Try .txt file
        let txt_path = p.with_extension("txt");
        if txt_path.exists() {
            if let Ok(content) = fs::read_to_string(&txt_path) {
                return Some(Self::parse_plain(&content));
            }
        }

        None
    }

    /// Parses LRC format string
    pub fn parse(content: &str) -> Self {
        let mut lyrics = Lyrics::default();
        let mut lines = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Check metadata tags
            if line.starts_with("[ti:") && line.ends_with(']') {
                lyrics.title = Some(line[4..line.len() - 1].trim().to_string());
                continue;
            }
            if line.starts_with("[ar:") && line.ends_with(']') {
                lyrics.artist = Some(line[4..line.len() - 1].trim().to_string());
                continue;
            }
            if line.starts_with("[al:") && line.ends_with(']') {
                lyrics.album = Some(line[4..line.len() - 1].trim().to_string());
                continue;
            }
            if line.starts_with("[offset:") && line.ends_with(']') {
                if let Ok(ms) = line[8..line.len() - 1].trim().parse::<f64>() {
                    lyrics.offset_ms = ms;
                }
                continue;
            }

            // Parse timestamp tags like [01:23.45]
            let mut remaining = line;
            let mut timestamps = Vec::new();

            while remaining.starts_with('[') {
                if let Some(close_idx) = remaining.find(']') {
                    let tag = &remaining[1..close_idx];
                    if let Some(t) = parse_lrc_timestamp(tag) {
                        timestamps.push(t);
                        remaining = &remaining[close_idx + 1..];
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let lyric_text = remaining.trim().to_string();
            if !timestamps.is_empty() {
                for ts in timestamps {
                    lines.push(LyricLine {
                        time_secs: (ts + lyrics.offset_ms / 1000.0).max(0.0),
                        text: lyric_text.clone(),
                    });
                }
            } else if !line.starts_with('[') {
                // Unsynced line
                lines.push(LyricLine {
                    time_secs: 0.0,
                    text: line.to_string(),
                });
            }
        }

        // Sort by timestamp
        lines.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap());
        lyrics.is_synced = lines.iter().any(|l| l.time_secs > 0.0);
        lyrics.lines = lines;
        lyrics
    }

    /// Parses plain unsynchronized text
    pub fn parse_plain(content: &str) -> Self {
        let lines = content
            .lines()
            .map(|l| LyricLine {
                time_secs: 0.0,
                text: l.trim().to_string(),
            })
            .collect();

        Lyrics {
            title: None,
            artist: None,
            album: None,
            offset_ms: 0.0,
            lines,
            is_synced: false,
        }
    }

    /// Finds the index of the active line for current playback time in seconds.
    pub fn current_line_index(&self, current_time_secs: f64) -> Option<usize> {
        if !self.is_synced || self.lines.is_empty() {
            return None;
        }

        let mut active_idx = None;
        for (i, line) in self.lines.iter().enumerate() {
            if current_time_secs >= line.time_secs {
                active_idx = Some(i);
            } else {
                break;
            }
        }
        active_idx
    }
}

/// Parses a timestamp like "01:23.45" or "01:23:45" or "01:23"
fn parse_lrc_timestamp(tag: &str) -> Option<f64> {
    let parts: Vec<&str> = tag.split(':').collect();
    if parts.len() == 2 {
        let mins: f64 = parts[0].trim().parse().ok()?;
        let secs: f64 = parts[1].trim().parse().ok()?;
        Some(mins * 60.0 + secs)
    } else if parts.len() == 3 {
        let hours: f64 = parts[0].trim().parse().ok()?;
        let mins: f64 = parts[1].trim().parse().ok()?;
        let secs: f64 = parts[2].trim().parse().ok()?;
        Some(hours * 3600.0 + mins * 60.0 + secs)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let lrc = r#"
[ti:Test Title]
[ar:Test Artist]
[00:05.50]Hello world
[00:10.00]Second line
"#;
        let parsed = Lyrics::parse(lrc);
        assert_eq!(parsed.title.as_deref(), Some("Test Title"));
        assert_eq!(parsed.artist.as_deref(), Some("Test Artist"));
        assert_eq!(parsed.lines.len(), 2);
        assert_eq!(parsed.current_line_index(6.0), Some(0));
        assert_eq!(parsed.current_line_index(12.0), Some(1));
    }
}
