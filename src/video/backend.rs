use super::vlc_binding::{
    LibvlcVideoCleanupCb, LibvlcVideoDisplayCb, LibvlcVideoFormatCb, LibvlcVideoLockCb,
    LibvlcVideoUnlockCb, TrackItem, VlcPaths, VlcSymbols,
};
use crate::audio::PlaybackState;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub struct VideoSharedBuffer {
    pub width: AtomicU32,
    pub height: AtomicU32,
    pub back_buffer: Mutex<Vec<u8>>,
    pub front_buffer: Mutex<Vec<u8>>,
    pub version: AtomicU64,
    pub has_new_frame: AtomicBool,
}

impl Default for VideoSharedBuffer {
    fn default() -> Self {
        Self {
            width: AtomicU32::new(0),
            height: AtomicU32::new(0),
            back_buffer: Mutex::new(Vec::new()),
            front_buffer: Mutex::new(Vec::new()),
            version: AtomicU64::new(0),
            has_new_frame: AtomicBool::new(false),
        }
    }
}

unsafe extern "C" fn video_format_cb(
    opaque: *mut *mut c_void,
    chroma: *mut c_char,
    width: *mut c_uint,
    height: *mut c_uint,
    pitches: *mut c_uint,
    lines: *mut c_uint,
) -> c_uint {
    if opaque.is_null() || (*opaque).is_null() {
        return 0;
    }
    let shared = &*((*opaque) as *const VideoSharedBuffer);
    let w = *width;
    let h = *height;

    // Set chroma to RGBA
    let chroma_slice = std::slice::from_raw_parts_mut(chroma as *mut u8, 4);
    chroma_slice.copy_from_slice(b"RGBA");

    *pitches = w * 4;
    *lines = h;

    shared.width.store(w, Ordering::SeqCst);
    shared.height.store(h, Ordering::SeqCst);

    let size = (w * h * 4) as usize;
    if let Ok(mut back) = shared.back_buffer.lock() {
        back.resize(size, 0);
    }
    if let Ok(mut front) = shared.front_buffer.lock() {
        front.resize(size, 0);
    }

    1
}

unsafe extern "C" fn video_cleanup_cb(_opaque: *mut c_void) {}

unsafe extern "C" fn video_lock_cb(opaque: *mut c_void, planes: *mut *mut c_void) -> *mut c_void {
    if opaque.is_null() || planes.is_null() {
        return std::ptr::null_mut();
    }
    let shared = &*(opaque as *const VideoSharedBuffer);
    let w = shared.width.load(Ordering::Relaxed);
    let h = shared.height.load(Ordering::Relaxed);
    let size = (w * h * 4) as usize;

    if let Ok(mut back) = shared.back_buffer.lock() {
        if back.len() != size && size > 0 {
            back.resize(size, 0);
        }
        let ptr = back.as_mut_ptr() as *mut c_void;
        *planes = ptr;
        ptr
    } else {
        std::ptr::null_mut()
    }
}

unsafe extern "C" fn video_unlock_cb(
    _opaque: *mut c_void,
    _picture: *mut c_void,
    _planes: *const *mut c_void,
) {
}

unsafe extern "C" fn video_display_cb(opaque: *mut c_void, _picture: *mut c_void) {
    if opaque.is_null() {
        return;
    }
    let shared = &*(opaque as *const VideoSharedBuffer);
    if let (Ok(mut back), Ok(mut front)) = (shared.back_buffer.lock(), shared.front_buffer.lock()) {
        std::mem::swap(&mut *back, &mut *front);
        shared.version.fetch_add(1, Ordering::Release);
        shared.has_new_frame.store(true, Ordering::Release);
    }
}

pub struct VideoBackend {
    symbols: Arc<VlcSymbols>,
    instance: *mut c_void,
    player: Mutex<Option<*mut c_void>>,
    shared_buffer: Box<VideoSharedBuffer>,
    current_path: Mutex<Option<String>>,
    is_open: AtomicBool,
}

// Raw pointers in libvlc are thread-safe across API calls
unsafe impl Send for VideoBackend {}
unsafe impl Sync for VideoBackend {}

impl VideoBackend {
    pub fn new(custom_path: Option<&str>) -> Result<Self, String> {
        let paths = VlcPaths::detect(custom_path)
            .ok_or_else(|| "LibVLC was not found on system. Please install VLC Media Player.".to_string())?;

        let symbols = VlcSymbols::load(&paths)?;

        let args = [
            CString::new("--no-video-title-show").unwrap(),
            CString::new("--quiet").unwrap(),
            CString::new("--vout=vmem").unwrap(),
        ];
        let arg_ptrs: Vec<*const c_char> = args.iter().map(|s| s.as_ptr()).collect();

        let instance = unsafe { (symbols.libvlc_new)(arg_ptrs.len() as c_int, arg_ptrs.as_ptr()) };
        if instance.is_null() {
            return Err("Failed to initialize LibVLC core instance.".to_string());
        }

        Ok(Self {
            symbols,
            instance,
            player: Mutex::new(None),
            shared_buffer: Box::new(VideoSharedBuffer::default()),
            current_path: Mutex::new(None),
            is_open: AtomicBool::new(false),
        })
    }

    pub fn version(&self) -> String {
        self.symbols.version_string()
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(Ordering::Relaxed)
    }

    pub fn current_path(&self) -> Option<String> {
        self.current_path.lock().unwrap().clone()
    }

    pub fn play(&self, path_or_url: &str) -> Result<(), String> {
        self.stop();

        let is_url = path_or_url.starts_with("http://")
            || path_or_url.starts_with("https://")
            || path_or_url.starts_with("rtsp://")
            || path_or_url.starts_with("mms://");

        let c_str = CString::new(path_or_url)
            .map_err(|e| format!("Invalid path/url string: {}", e))?;

        let media = unsafe {
            if is_url {
                (self.symbols.libvlc_media_new_location)(self.instance, c_str.as_ptr())
            } else {
                (self.symbols.libvlc_media_new_path)(self.instance, c_str.as_ptr())
            }
        };

        if media.is_null() {
            return Err(format!("LibVLC could not open media: {}", path_or_url));
        }

        let player = unsafe { (self.symbols.libvlc_media_player_new_from_media)(media) };
        unsafe { (self.symbols.libvlc_media_release)(media) };

        if player.is_null() {
            return Err("Failed to create LibVLC media player.".to_string());
        }

        // Setup memory rendering callbacks
        let shared_ptr = &*self.shared_buffer as *const VideoSharedBuffer as *mut c_void;
        unsafe {
            (self.symbols.libvlc_video_set_format_callbacks)(
                player,
                video_format_cb as LibvlcVideoFormatCb,
                Some(video_cleanup_cb as LibvlcVideoCleanupCb),
            );
            (self.symbols.libvlc_video_set_callbacks)(
                player,
                video_lock_cb as LibvlcVideoLockCb,
                Some(video_unlock_cb as LibvlcVideoUnlockCb),
                Some(video_display_cb as LibvlcVideoDisplayCb),
                shared_ptr,
            );
        }

        // Start playback
        let res = unsafe { (self.symbols.libvlc_media_player_play)(player) };
        if res != 0 {
            unsafe { (self.symbols.libvlc_media_player_release)(player) };
            return Err("Failed to start LibVLC media player.".to_string());
        }

        *self.player.lock().unwrap() = Some(player);
        *self.current_path.lock().unwrap() = Some(path_or_url.to_string());
        self.is_open.store(true, Ordering::SeqCst);

        Ok(())
    }

    pub fn pause(&self) {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_media_player_pause)(player) };
        }
    }

    pub fn resume(&self) {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_media_player_play)(player) };
        }
    }

    pub fn toggle_play_pause(&self) {
        match self.state() {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.resume(),
            PlaybackState::Stopped => {
                if let Some(p) = self.current_path() {
                    let _ = self.play(&p);
                }
            }
            _ => {}
        }
    }

    pub fn stop(&self) {
        let mut guard = self.player.lock().unwrap();
        if let Some(player) = guard.take() {
            unsafe {
                (self.symbols.libvlc_media_player_stop)(player);
                (self.symbols.libvlc_media_player_release)(player);
            }
        }
        self.is_open.store(false, Ordering::SeqCst);
    }

    pub fn seek(&self, time_secs: f64) {
        if let Some(player) = *self.player.lock().unwrap() {
            let ms = (time_secs * 1000.0).max(0.0) as i64;
            unsafe { (self.symbols.libvlc_media_player_set_time)(player, ms) };
        }
    }

    pub fn set_position(&self, ratio: f32) {
        if let Some(player) = *self.player.lock().unwrap() {
            let r = ratio.clamp(0.0, 1.0);
            unsafe { (self.symbols.libvlc_media_player_set_position)(player, r) };
        }
    }

    pub fn get_position(&self) -> f32 {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_media_player_get_position)(player) }
        } else {
            0.0
        }
    }

    pub fn get_time(&self) -> f64 {
        if let Some(player) = *self.player.lock().unwrap() {
            let ms = unsafe { (self.symbols.libvlc_media_player_get_time)(player) };
            if ms > 0 {
                ms as f64 / 1000.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn get_duration(&self) -> f64 {
        if let Some(player) = *self.player.lock().unwrap() {
            let ms = unsafe { (self.symbols.libvlc_media_player_get_length)(player) };
            if ms > 0 {
                ms as f64 / 1000.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn set_volume(&self, volume: f32) {
        if let Some(player) = *self.player.lock().unwrap() {
            // VLC volume: 0 to 200 (100 is 100%)
            let vol_int = (volume * 100.0).clamp(0.0, 200.0).round() as c_int;
            unsafe { (self.symbols.libvlc_audio_set_volume)(player, vol_int) };
        }
    }

    pub fn set_mute(&self, mute: bool) {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_audio_set_mute)(player, if mute { 1 } else { 0 }) };
        }
    }

    pub fn set_speed(&self, speed: f32) {
        if let Some(player) = *self.player.lock().unwrap() {
            let spd = speed.clamp(0.25, 4.0);
            unsafe { (self.symbols.libvlc_media_player_set_rate)(player, spd) };
        }
    }

    pub fn state(&self) -> PlaybackState {
        if let Some(player) = *self.player.lock().unwrap() {
            let st = unsafe { (self.symbols.libvlc_media_player_get_state)(player) };
            match st {
                3 => PlaybackState::Playing,
                4 => PlaybackState::Paused,
                2 => PlaybackState::Buffering,
                6 => PlaybackState::Stopped, // Ended
                _ => PlaybackState::Stopped,
            }
        } else {
            PlaybackState::Stopped
        }
    }

    pub fn is_ended(&self) -> bool {
        if let Some(player) = *self.player.lock().unwrap() {
            let st = unsafe { (self.symbols.libvlc_media_player_get_state)(player) };
            st == 6 // Ended
        } else {
            false
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (
            self.shared_buffer.width.load(Ordering::Relaxed),
            self.shared_buffer.height.load(Ordering::Relaxed),
        )
    }

    /// Read subtitle tracks (SPU)
    pub fn get_subtitles(&self) -> Vec<TrackItem> {
        let mut list = Vec::new();
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe {
                let p_desc = (self.symbols.libvlc_video_get_spu_description)(player);
                let mut curr = p_desc;
                while !curr.is_null() {
                    let id = (*curr).id;
                    let name = if (*curr).name.is_null() {
                        "Unknown".to_string()
                    } else {
                        CStr::from_ptr((*curr).name).to_string_lossy().to_string()
                    };
                    list.push(TrackItem { id, name });
                    curr = (*curr).next;
                }
                if !p_desc.is_null() {
                    (self.symbols.libvlc_track_description_list_release)(p_desc);
                }
            }
        }
        list
    }

    pub fn get_current_subtitle(&self) -> i32 {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_video_get_spu)(player) }
        } else {
            -1
        }
    }

    pub fn set_subtitle(&self, id: i32) {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_video_set_spu)(player, id) };
        }
    }

    pub fn add_subtitle_file(&self, path: &Path) -> bool {
        if let Some(player) = *self.player.lock().unwrap() {
            if let Ok(c_path) = CString::new(path.to_string_lossy().to_string()) {
                let res = unsafe { (self.symbols.libvlc_video_set_subtitle_file)(player, c_path.as_ptr()) };
                return res == 1 || res == 0;
            }
        }
        false
    }

    /// Read audio tracks
    pub fn get_audio_tracks(&self) -> Vec<TrackItem> {
        let mut list = Vec::new();
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe {
                let p_desc = (self.symbols.libvlc_audio_get_track_description)(player);
                let mut curr = p_desc;
                while !curr.is_null() {
                    let id = (*curr).id;
                    let name = if (*curr).name.is_null() {
                        "Unknown".to_string()
                    } else {
                        CStr::from_ptr((*curr).name).to_string_lossy().to_string()
                    };
                    list.push(TrackItem { id, name });
                    curr = (*curr).next;
                }
                if !p_desc.is_null() {
                    (self.symbols.libvlc_track_description_list_release)(p_desc);
                }
            }
        }
        list
    }

    pub fn get_current_audio_track(&self) -> i32 {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_audio_get_track)(player) }
        } else {
            -1
        }
    }

    pub fn set_audio_track(&self, id: i32) {
        if let Some(player) = *self.player.lock().unwrap() {
            unsafe { (self.symbols.libvlc_audio_set_track)(player, id) };
        }
    }

    pub fn take_snapshot(&self, path: &Path) -> bool {
        if let Some(player) = *self.player.lock().unwrap() {
            if let Ok(c_path) = CString::new(path.to_string_lossy().to_string()) {
                let res = unsafe {
                    (self.symbols.libvlc_video_take_snapshot)(
                        player,
                        0,
                        c_path.as_ptr(),
                        0,
                        0,
                    )
                };
                return res == 0;
            }
        }
        false
    }

    /// Check if a newly decoded frame is ready, and copy its dimensions & RGBA bytes
    pub fn poll_new_frame(&self) -> Option<(u32, u32, Vec<u8>)> {
        if self.shared_buffer.has_new_frame.swap(false, Ordering::AcqRel) {
            let w = self.shared_buffer.width.load(Ordering::Relaxed);
            let h = self.shared_buffer.height.load(Ordering::Relaxed);
            if w > 0 && h > 0 {
                if let Ok(front) = self.shared_buffer.front_buffer.lock() {
                    if front.len() == (w * h * 4) as usize {
                        return Some((w, h, front.clone()));
                    }
                }
            }
        }
        None
    }
}

impl Drop for VideoBackend {
    fn drop(&mut self) {
        self.stop();
        if !self.instance.is_null() {
            unsafe { (self.symbols.libvlc_release)(self.instance) };
        }
    }
}
