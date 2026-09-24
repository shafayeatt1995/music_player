pub mod track;

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
pub use track::Track;
use walkdir::WalkDir;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    Single,
}

impl RepeatMode {
    pub fn cycle(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::Single,
            RepeatMode::Single => RepeatMode::Off,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            RepeatMode::Off => "Repeat: Off",
            RepeatMode::All => "Repeat: All",
            RepeatMode::Single => "Repeat: 1",
        }
    }
}

pub struct Playlist {
    tracks: Vec<Track>,
    current_index: Option<usize>,
    shuffle_indices: Vec<usize>,
    shuffle_pos: usize,
    pub shuffle_enabled: bool,
    pub repeat_mode: RepeatMode,
    pub search_query: String,
    next_id: u64,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            shuffle_indices: Vec::new(),
            shuffle_pos: 0,
            shuffle_enabled: false,
            repeat_mode: RepeatMode::Off,
            search_query: String::new(),
            next_id: 1,
        }
    }
}

impl Playlist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn tracks_mut(&mut self) -> &mut [Track] {
        &mut self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_index.and_then(|idx| self.tracks.get(idx))
    }

    pub fn current_track_mut(&mut self) -> Option<&mut Track> {
        self.current_index.and_then(|idx| self.tracks.get_mut(idx))
    }

    pub fn add_file(&mut self, path: &Path) {
        if is_supported_media_extension(path) {
            let track = Track::new_from_path(self.next_id, path);
            self.next_id += 1;
            self.tracks.push(track);
            self.rebuild_shuffle();
        }
    }

    pub fn add_directory(&mut self, dir_path: &Path) {
        for entry in WalkDir::new(dir_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if is_supported_media_extension(path) {
                let track = Track::new_from_path(self.next_id, path);
                self.next_id += 1;
                self.tracks.push(track);
            }
        }
        self.rebuild_shuffle();
    }

    pub fn add_stream(&mut self, url: &str, title: Option<&str>) {
        let track = Track::new_stream(self.next_id, url, title);
        self.next_id += 1;
        self.tracks.push(track);
        self.rebuild_shuffle();
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
            if let Some(cur) = self.current_index {
                if cur == index {
                    if self.tracks.is_empty() {
                        self.current_index = None;
                    } else if cur >= self.tracks.len() {
                        self.current_index = Some(self.tracks.len() - 1);
                    }
                } else if cur > index {
                    self.current_index = Some(cur - 1);
                }
            }
            self.rebuild_shuffle();
        }
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
        self.shuffle_indices.clear();
        self.shuffle_pos = 0;
    }

    pub fn play_index(&mut self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            self.current_index = Some(index);
            if self.shuffle_enabled {
                if let Some(pos) = self.shuffle_indices.iter().position(|&i| i == index) {
                    self.shuffle_pos = pos;
                }
            }
            self.tracks.get(index)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<(usize, &Track)> {
        if self.tracks.is_empty() {
            return None;
        }

        if self.repeat_mode == RepeatMode::Single {
            if let Some(cur) = self.current_index {
                return Some((cur, &self.tracks[cur]));
            }
        }

        if self.shuffle_enabled && !self.shuffle_indices.is_empty() {
            self.shuffle_pos += 1;
            if self.shuffle_pos >= self.shuffle_indices.len() {
                if self.repeat_mode == RepeatMode::All {
                    self.rebuild_shuffle();
                    self.shuffle_pos = 0;
                } else {
                    return None;
                }
            }
            let next_idx = self.shuffle_indices[self.shuffle_pos];
            self.current_index = Some(next_idx);
            return Some((next_idx, &self.tracks[next_idx]));
        }

        let next_idx = match self.current_index {
            Some(cur) => {
                if cur + 1 < self.tracks.len() {
                    cur + 1
                } else if self.repeat_mode == RepeatMode::All {
                    0
                } else {
                    return None;
                }
            }
            None => 0,
        };

        self.current_index = Some(next_idx);
        Some((next_idx, &self.tracks[next_idx]))
    }

    pub fn previous(&mut self) -> Option<(usize, &Track)> {
        if self.tracks.is_empty() {
            return None;
        }

        if self.shuffle_enabled && !self.shuffle_indices.is_empty() {
            if self.shuffle_pos > 0 {
                self.shuffle_pos -= 1;
            } else if self.repeat_mode == RepeatMode::All {
                self.shuffle_pos = self.shuffle_indices.len() - 1;
            }
            let prev_idx = self.shuffle_indices[self.shuffle_pos];
            self.current_index = Some(prev_idx);
            return Some((prev_idx, &self.tracks[prev_idx]));
        }

        let prev_idx = match self.current_index {
            Some(cur) => {
                if cur > 0 {
                    cur - 1
                } else if self.repeat_mode == RepeatMode::All {
                    self.tracks.len() - 1
                } else {
                    0
                }
            }
            None => 0,
        };

        self.current_index = Some(prev_idx);
        Some((prev_idx, &self.tracks[prev_idx]))
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle_enabled = !self.shuffle_enabled;
        if self.shuffle_enabled {
            self.rebuild_shuffle();
        }
    }

    pub fn rebuild_shuffle(&mut self) {
        let n = self.tracks.len();
        if n == 0 {
            self.shuffle_indices.clear();
            self.shuffle_pos = 0;
            return;
        }

        self.shuffle_indices = (0..n).collect();
        // Fisher-Yates shuffle using pseudo-random LCG to avoid extra rng dependencies
        let mut seed = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(123456789)
            & 0xFFFF_FFFF) as u64;

        for i in (1..n).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = ((seed >> 32) as usize) % (i + 1);
            self.shuffle_indices.swap(i, j);
        }

        if let Some(cur) = self.current_index {
            if let Some(pos) = self.shuffle_indices.iter().position(|&i| i == cur) {
                self.shuffle_indices.swap(0, pos);
                self.shuffle_pos = 0;
            }
        }
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return (0..self.tracks.len()).collect();
        }

        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.title.to_lowercase().contains(&query)
                    || t.artist.to_lowercase().contains(&query)
                    || t.album.to_lowercase().contains(&query)
                    || t.path_or_url.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn total_duration(&self) -> f64 {
        self.tracks.iter().map(|t| t.duration).sum()
    }

    /// Import M3U or M3U8 playlist file
    pub fn import_m3u(&mut self, path: &Path) -> Result<usize, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open M3U: {}", e))?;
        let reader = BufReader::new(file);
        let base_dir = path.parent().unwrap_or_else(|| Path::new("."));

        let mut count = 0;
        let mut next_title: Option<String> = None;

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Read error: {}", e))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed.starts_with("#EXTINF:") {
                // e.g. #EXTINF:123,Artist - Title
                if let Some(comma_pos) = trimmed.find(',') {
                    next_title = Some(trimmed[comma_pos + 1..].trim().to_string());
                }
                continue;
            }

            if trimmed.starts_with('#') {
                continue;
            }

            // Path or URL
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                let track = Track::new_stream(self.next_id, trimmed, next_title.as_deref());
                self.next_id += 1;
                self.tracks.push(track);
                count += 1;
            } else {
                let item_path = PathBuf::from(trimmed);
                let full_path = if item_path.is_absolute() {
                    item_path
                } else {
                    base_dir.join(item_path)
                };

                if full_path.exists() {
                    let mut track = Track::new_from_path(self.next_id, &full_path);
                    if let Some(title) = next_title.take() {
                        track.title = title;
                    }
                    self.next_id += 1;
                    self.tracks.push(track);
                    count += 1;
                }
            }
            next_title = None;
        }

        self.rebuild_shuffle();
        Ok(count)
    }

    /// Export current playlist to M3U file
    pub fn export_m3u(&self, path: &Path) -> Result<(), String> {
        let mut file = File::create(path).map_err(|e| format!("Failed to write M3U: {}", e))?;
        writeln!(file, "#EXTM3U").map_err(|e| e.to_string())?;

        for track in &self.tracks {
            let dur = track.duration.round() as i64;
            writeln!(file, "#EXTINF:{},{} - {}", dur, track.artist, track.title)
                .map_err(|e| e.to_string())?;
            writeln!(file, "{}", track.path_or_url).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

pub fn is_supported_audio_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    matches!(
        ext.as_deref(),
        Some("mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" | "alac" | "opus" | "aiff" | "caf" | "wma")
    )
}

pub fn is_supported_video_extension(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    matches!(
        ext.as_deref(),
        Some(
            "mp4" | "m4v" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "ts" | "m2ts" | "vob"
                | "mpg" | "mpeg" | "3gp" | "ogv" | "divx" | "rmvb" | "asf"
        )
    )
}

pub fn is_supported_media_extension(path: &Path) -> bool {
    is_supported_audio_extension(path) || is_supported_video_extension(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_video_extensions() {
        assert!(is_supported_video_extension(Path::new("movie.mp4")));
        assert!(is_supported_video_extension(Path::new("video.mkv")));
        assert!(is_supported_video_extension(Path::new("clip.AVI")));
        assert!(is_supported_video_extension(Path::new("recording.MOV")));
        assert!(is_supported_video_extension(Path::new("stream.webm")));
        assert!(is_supported_video_extension(Path::new("flash.flv")));
        assert!(is_supported_video_extension(Path::new("media.wmv")));
        assert!(is_supported_video_extension(Path::new("broadcast.ts")));
        assert!(is_supported_video_extension(Path::new("dvd.vob")));

        assert!(!is_supported_video_extension(Path::new("song.mp3")));
        assert!(!is_supported_video_extension(Path::new("document.pdf")));
    }

    #[test]
    fn test_supported_media_extensions() {
        assert!(is_supported_media_extension(Path::new("song.mp3")));
        assert!(is_supported_media_extension(Path::new("audio.flac")));
        assert!(is_supported_media_extension(Path::new("movie.mp4")));
        assert!(is_supported_media_extension(Path::new("film.mkv")));
        assert!(!is_supported_media_extension(Path::new("text.txt")));
    }
}
