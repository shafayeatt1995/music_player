use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: u64,
    pub path_or_url: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub is_stream: bool,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
}

impl Track {
    pub fn new_from_path(id: u64, path: &Path) -> Self {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Track");

        // Parse default title by stripping extension
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(filename)
            .to_string();

        Self {
            id,
            path_or_url: path.to_string_lossy().to_string(),
            title,
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            duration: 0.0,
            is_stream: false,
            codec: None,
            bitrate: None,
            sample_rate: None,
            channels: None,
        }
    }

    pub fn new_stream(id: u64, url: &str, name: Option<&str>) -> Self {
        let title = name.unwrap_or_else(|| {
            if url.contains('/') {
                url.split('/').last().unwrap_or(url)
            } else {
                url
            }
        });

        Self {
            id,
            path_or_url: url.to_string(),
            title: title.to_string(),
            artist: "Internet Stream".to_string(),
            album: "Live Radio".to_string(),
            duration: 0.0,
            is_stream: true,
            codec: Some("Live Stream".to_string()),
            bitrate: None,
            sample_rate: None,
            channels: None,
        }
    }

    pub fn is_local_file(&self) -> bool {
        !self.is_stream && Path::new(&self.path_or_url).exists()
    }

    pub fn is_video(&self) -> bool {
        !self.is_stream && super::is_supported_video_extension(Path::new(&self.path_or_url))
    }
}
