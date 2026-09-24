pub mod backend;
pub mod vlc_binding;

pub use backend::VideoBackend;
pub use vlc_binding::TrackItem;

#[cfg(test)]
mod tests {
    use super::*;
    use super::vlc_binding::VlcPaths;

    #[test]
    fn test_vlc_paths_detect() {
        let paths = VlcPaths::detect(None);
        if let Some(ref p) = paths {
            assert!(p.libvlc.exists(), "Detected libvlc path should exist");
        }
    }

    #[test]
    fn test_video_backend_lifecycle() {
        if let Ok(backend) = VideoBackend::new(None) {
            let ver = backend.version();
            assert!(!ver.is_empty(), "Version should not be empty");
            assert_eq!(backend.state(), crate::audio::PlaybackState::Stopped);
            assert_eq!(backend.is_open(), false);
        }
    }

    #[test]
    fn test_video_playback_real_file() {
        let test_video = "/Users/shafayetal-anik/Downloads/Video/Awarapan 2 (2026) Hindi 1080p HDTC x265 HEVC 10bit AAC - HDHub4u.mkv";
        if std::path::Path::new(test_video).exists() {
            if let Ok(backend) = VideoBackend::new(None) {
                assert!(backend.play(test_video).is_ok());
                std::thread::sleep(std::time::Duration::from_millis(1500));

                assert!(backend.is_open());
                let (w, h) = backend.dimensions();
                assert!(w > 0 && h > 0, "Dimensions should be populated, got {}x{}", w, h);

                let frame = backend.poll_new_frame();
                assert!(frame.is_some(), "Should have received decoded frame");
                if let Some((fw, fh, pixels)) = frame {
                    assert_eq!(fw, w);
                    assert_eq!(fh, h);
                    assert_eq!(pixels.len(), (w * h * 4) as usize);
                }

                let sub_tracks = backend.get_subtitles();
                println!("Subtitles found: {}", sub_tracks.len());

                let audio_tracks = backend.get_audio_tracks();
                println!("Audio tracks found: {}", audio_tracks.len());

                backend.stop();
                assert_eq!(backend.is_open(), false);
            }
        }
    }
}
