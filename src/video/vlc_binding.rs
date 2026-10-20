use libloading::{Library, Symbol};
use std::ffi::{c_void, CStr};
use std::os::raw::{c_char, c_int, c_uint};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[repr(C)]
pub struct LibvlcTrackDescription {
    pub id: c_int,
    pub name: *mut c_char,
    pub next: *mut LibvlcTrackDescription,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackItem {
    pub id: i32,
    pub name: String,
}

pub type LibvlcVideoFormatCb = unsafe extern "C" fn(
    opaque: *mut *mut c_void,
    chroma: *mut c_char,
    width: *mut c_uint,
    height: *mut c_uint,
    pitches: *mut c_uint,
    lines: *mut c_uint,
) -> c_uint;

pub type LibvlcVideoCleanupCb = unsafe extern "C" fn(opaque: *mut c_void);
pub type LibvlcVideoLockCb = unsafe extern "C" fn(opaque: *mut c_void, planes: *mut *mut c_void) -> *mut c_void;
pub type LibvlcVideoUnlockCb = unsafe extern "C" fn(opaque: *mut c_void, picture: *mut c_void, planes: *const *mut c_void);
pub type LibvlcVideoDisplayCb = unsafe extern "C" fn(opaque: *mut c_void, picture: *mut c_void);

#[derive(Clone, Debug)]
pub struct VlcPaths {
    pub libvlc: PathBuf,
    pub libvlccore: Option<PathBuf>,
    pub plugins_dir: Option<PathBuf>,
}

impl VlcPaths {
    pub fn detect(custom_path: Option<&str>) -> Option<Self> {
        if let Some(custom) = custom_path {
            let p = PathBuf::from(custom);
            if p.exists() {
                let dir = p.parent().unwrap_or(Path::new(""));
                let core = dir.join("libvlccore.dylib");
                let core_path = if core.exists() { Some(core) } else { None };
                let plugins = dir.parent().map(|p| p.join("plugins"));
                return Some(Self {
                    libvlc: p,
                    libvlccore: core_path,
                    plugins_dir: plugins.filter(|p| p.exists()),
                });
            }
        }

        #[cfg(target_os = "macos")]
        {
            let candidates = [
                (
                    "/Applications/VLC.app/Contents/MacOS/lib/libvlc.dylib",
                    Some("/Applications/VLC.app/Contents/MacOS/lib/libvlccore.dylib"),
                    Some("/Applications/VLC.app/Contents/MacOS/plugins"),
                ),
                (
                    "/opt/homebrew/lib/libvlc.dylib",
                    Some("/opt/homebrew/lib/libvlccore.dylib"),
                    Some("/opt/homebrew/lib/vlc/plugins"),
                ),
                (
                    "/usr/local/lib/libvlc.dylib",
                    Some("/usr/local/lib/libvlccore.dylib"),
                    Some("/usr/local/lib/vlc/plugins"),
                ),
            ];

            for (vlc, core, plugins) in candidates {
                let vlc_p = PathBuf::from(vlc);
                if vlc_p.exists() {
                    return Some(Self {
                        libvlc: vlc_p,
                        libvlccore: core.map(PathBuf::from).filter(|p| p.exists()),
                        plugins_dir: plugins.map(PathBuf::from).filter(|p| p.exists()),
                    });
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            let candidates: [(&str, Option<&str>, Option<&str>); 2] = [
                (
                    "C:\\Program Files\\VideoLAN\\VLC\\libvlc.dll",
                    None,
                    Some("C:\\Program Files\\VideoLAN\\VLC\\plugins"),
                ),
                (
                    "C:\\Program Files (x86)\\VideoLAN\\VLC\\libvlc.dll",
                    None,
                    Some("C:\\Program Files (x86)\\VideoLAN\\VLC\\plugins"),
                ),
            ];

            for (vlc, core, plugins) in candidates {
                let vlc_p = PathBuf::from(vlc);
                if vlc_p.exists() {
                    return Some(Self {
                        libvlc: vlc_p,
                        libvlccore: core.map(PathBuf::from),
                        plugins_dir: plugins.map(PathBuf::from).filter(|p| p.exists()),
                    });
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates: [(&str, Option<&str>, Option<&str>); 4] = [
                (
                    "/usr/lib/x86_64-linux-gnu/libvlc.so.5",
                    None,
                    Some("/usr/lib/x86_64-linux-gnu/vlc/plugins"),
                ),
                (
                    "/usr/lib/libvlc.so.5",
                    None,
                    Some("/usr/lib/vlc/plugins"),
                ),
                (
                    "/usr/lib/x86_64-linux-gnu/libvlc.so",
                    None,
                    Some("/usr/lib/x86_64-linux-gnu/vlc/plugins"),
                ),
                (
                    "/usr/lib/libvlc.so",
                    None,
                    Some("/usr/lib/vlc/plugins"),
                ),
            ];

            for (vlc, core, plugins) in candidates {
                let vlc_p = PathBuf::from(vlc);
                if vlc_p.exists() {
                    return Some(Self {
                        libvlc: vlc_p,
                        libvlccore: core.map(PathBuf::from),
                        plugins_dir: plugins.map(PathBuf::from).filter(|p| p.exists()),
                    });
                }
            }
        }

        None
    }
}

pub struct VlcSymbols {
    _vlccore_lib: Option<Library>,
    _vlc_lib: Library,

    pub libvlc_new: unsafe extern "C" fn(argc: c_int, argv: *const *const c_char) -> *mut c_void,
    pub libvlc_release: unsafe extern "C" fn(p_instance: *mut c_void),
    pub libvlc_get_version: unsafe extern "C" fn() -> *const c_char,
    pub libvlc_media_new_path: unsafe extern "C" fn(p_instance: *mut c_void, path: *const c_char) -> *mut c_void,
    pub libvlc_media_new_location: unsafe extern "C" fn(p_instance: *mut c_void, psz_mrl: *const c_char) -> *mut c_void,
    pub libvlc_media_release: unsafe extern "C" fn(p_md: *mut c_void),
    pub libvlc_media_player_new_from_media: unsafe extern "C" fn(p_md: *mut c_void) -> *mut c_void,
    pub libvlc_media_player_release: unsafe extern "C" fn(p_mi: *mut c_void),
    pub libvlc_media_player_play: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_media_player_pause: unsafe extern "C" fn(p_mi: *mut c_void),
    pub libvlc_media_player_stop: unsafe extern "C" fn(p_mi: *mut c_void),
    pub libvlc_media_player_is_playing: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_media_player_get_state: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_media_player_get_length: unsafe extern "C" fn(p_mi: *mut c_void) -> i64,
    pub libvlc_media_player_get_time: unsafe extern "C" fn(p_mi: *mut c_void) -> i64,
    pub libvlc_media_player_set_time: unsafe extern "C" fn(p_mi: *mut c_void, i_time: i64),
    pub libvlc_media_player_get_position: unsafe extern "C" fn(p_mi: *mut c_void) -> f32,
    pub libvlc_media_player_set_position: unsafe extern "C" fn(p_mi: *mut c_void, f_pos: f32),
    pub libvlc_media_player_set_rate: unsafe extern "C" fn(p_mi: *mut c_void, rate: f32) -> c_int,
    pub libvlc_media_player_get_rate: unsafe extern "C" fn(p_mi: *mut c_void) -> f32,
    pub libvlc_audio_get_volume: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_audio_set_volume: unsafe extern "C" fn(p_mi: *mut c_void, i_volume: c_int) -> c_int,
    pub libvlc_audio_get_mute: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_audio_set_mute: unsafe extern "C" fn(p_mi: *mut c_void, status: c_int),
    pub libvlc_video_set_callbacks: unsafe extern "C" fn(
        p_mi: *mut c_void,
        lock: LibvlcVideoLockCb,
        unlock: Option<LibvlcVideoUnlockCb>,
        display: Option<LibvlcVideoDisplayCb>,
        opaque: *mut c_void,
    ),
    pub libvlc_video_set_format_callbacks: unsafe extern "C" fn(
        p_mi: *mut c_void,
        setup: LibvlcVideoFormatCb,
        cleanup: Option<LibvlcVideoCleanupCb>,
    ),
    pub libvlc_video_get_size: unsafe extern "C" fn(
        p_mi: *mut c_void,
        num: c_uint,
        px: *mut c_uint,
        py: *mut c_uint,
    ) -> c_int,
    pub libvlc_video_get_spu_count: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_video_get_spu_description: unsafe extern "C" fn(p_mi: *mut c_void) -> *mut LibvlcTrackDescription,
    pub libvlc_video_get_spu: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_video_set_spu: unsafe extern "C" fn(p_mi: *mut c_void, i_spu: c_int) -> c_int,
    pub libvlc_video_set_subtitle_file: unsafe extern "C" fn(p_mi: *mut c_void, psz_subtitle: *const c_char) -> c_int,
    pub libvlc_audio_get_track_count: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_audio_get_track_description: unsafe extern "C" fn(p_mi: *mut c_void) -> *mut LibvlcTrackDescription,
    pub libvlc_audio_get_track: unsafe extern "C" fn(p_mi: *mut c_void) -> c_int,
    pub libvlc_audio_set_track: unsafe extern "C" fn(p_mi: *mut c_void, i_track: c_int) -> c_int,
    pub libvlc_track_description_list_release: unsafe extern "C" fn(p_track_description: *mut LibvlcTrackDescription),
    pub libvlc_video_take_snapshot: unsafe extern "C" fn(
        p_mi: *mut c_void,
        num: c_uint,
        psz_filepath: *const c_char,
        i_width: c_uint,
        i_height: c_uint,
    ) -> c_int,
}

impl VlcSymbols {
    pub fn load(paths: &VlcPaths) -> Result<Arc<Self>, String> {
        // Set VLC_PLUGIN_PATH if plugins directory exists
        if let Some(ref plugins) = paths.plugins_dir {
            if plugins.exists() {
                std::env::set_var("VLC_PLUGIN_PATH", plugins.to_string_lossy().to_string());
            }
        }

        unsafe {
            let vlccore_lib = if let Some(ref core_path) = paths.libvlccore {
                match Library::new(core_path) {
                    Ok(lib) => Some(lib),
                    Err(e) => {
                        eprintln!("Warning: Failed to preload libvlccore: {}", e);
                        None
                    }
                }
            } else {
                None
            };

            let vlc_lib = Library::new(&paths.libvlc)
                .map_err(|e| format!("Failed to load libvlc dynamically from {:?}: {}", paths.libvlc, e))?;

            macro_rules! get_sym {
                ($name:ident, $type:ty) => {{
                    let sym: Symbol<$type> = vlc_lib
                        .get(stringify!($name).as_bytes())
                        .map_err(|e| format!("Symbol {} not found in libvlc: {}", stringify!($name), e))?;
                    *sym
                }};
            }

            let s_libvlc_new = get_sym!(libvlc_new, _);
            let s_libvlc_release = get_sym!(libvlc_release, _);
            let s_libvlc_get_version = get_sym!(libvlc_get_version, _);
            let s_libvlc_media_new_path = get_sym!(libvlc_media_new_path, _);
            let s_libvlc_media_new_location = get_sym!(libvlc_media_new_location, _);
            let s_libvlc_media_release = get_sym!(libvlc_media_release, _);
            let s_libvlc_media_player_new_from_media = get_sym!(libvlc_media_player_new_from_media, _);
            let s_libvlc_media_player_release = get_sym!(libvlc_media_player_release, _);
            let s_libvlc_media_player_play = get_sym!(libvlc_media_player_play, _);
            let s_libvlc_media_player_pause = get_sym!(libvlc_media_player_pause, _);
            let s_libvlc_media_player_stop = get_sym!(libvlc_media_player_stop, _);
            let s_libvlc_media_player_is_playing = get_sym!(libvlc_media_player_is_playing, _);
            let s_libvlc_media_player_get_state = get_sym!(libvlc_media_player_get_state, _);
            let s_libvlc_media_player_get_length = get_sym!(libvlc_media_player_get_length, _);
            let s_libvlc_media_player_get_time = get_sym!(libvlc_media_player_get_time, _);
            let s_libvlc_media_player_set_time = get_sym!(libvlc_media_player_set_time, _);
            let s_libvlc_media_player_get_position = get_sym!(libvlc_media_player_get_position, _);
            let s_libvlc_media_player_set_position = get_sym!(libvlc_media_player_set_position, _);
            let s_libvlc_media_player_set_rate = get_sym!(libvlc_media_player_set_rate, _);
            let s_libvlc_media_player_get_rate = get_sym!(libvlc_media_player_get_rate, _);
            let s_libvlc_audio_get_volume = get_sym!(libvlc_audio_get_volume, _);
            let s_libvlc_audio_set_volume = get_sym!(libvlc_audio_set_volume, _);
            let s_libvlc_audio_get_mute = get_sym!(libvlc_audio_get_mute, _);
            let s_libvlc_audio_set_mute = get_sym!(libvlc_audio_set_mute, _);
            let s_libvlc_video_set_callbacks = get_sym!(libvlc_video_set_callbacks, _);
            let s_libvlc_video_set_format_callbacks = get_sym!(libvlc_video_set_format_callbacks, _);
            let s_libvlc_video_get_size = get_sym!(libvlc_video_get_size, _);
            let s_libvlc_video_get_spu_count = get_sym!(libvlc_video_get_spu_count, _);
            let s_libvlc_video_get_spu_description = get_sym!(libvlc_video_get_spu_description, _);
            let s_libvlc_video_get_spu = get_sym!(libvlc_video_get_spu, _);
            let s_libvlc_video_set_spu = get_sym!(libvlc_video_set_spu, _);
            let s_libvlc_video_set_subtitle_file = get_sym!(libvlc_video_set_subtitle_file, _);
            let s_libvlc_audio_get_track_count = get_sym!(libvlc_audio_get_track_count, _);
            let s_libvlc_audio_get_track_description = get_sym!(libvlc_audio_get_track_description, _);
            let s_libvlc_audio_get_track = get_sym!(libvlc_audio_get_track, _);
            let s_libvlc_audio_set_track = get_sym!(libvlc_audio_set_track, _);
            let s_libvlc_track_description_list_release = get_sym!(libvlc_track_description_list_release, _);
            let s_libvlc_video_take_snapshot = get_sym!(libvlc_video_take_snapshot, _);

            let syms = Self {
                _vlccore_lib: vlccore_lib,
                _vlc_lib: vlc_lib,

                libvlc_new: s_libvlc_new,
                libvlc_release: s_libvlc_release,
                libvlc_get_version: s_libvlc_get_version,
                libvlc_media_new_path: s_libvlc_media_new_path,
                libvlc_media_new_location: s_libvlc_media_new_location,
                libvlc_media_release: s_libvlc_media_release,
                libvlc_media_player_new_from_media: s_libvlc_media_player_new_from_media,
                libvlc_media_player_release: s_libvlc_media_player_release,
                libvlc_media_player_play: s_libvlc_media_player_play,
                libvlc_media_player_pause: s_libvlc_media_player_pause,
                libvlc_media_player_stop: s_libvlc_media_player_stop,
                libvlc_media_player_is_playing: s_libvlc_media_player_is_playing,
                libvlc_media_player_get_state: s_libvlc_media_player_get_state,
                libvlc_media_player_get_length: s_libvlc_media_player_get_length,
                libvlc_media_player_get_time: s_libvlc_media_player_get_time,
                libvlc_media_player_set_time: s_libvlc_media_player_set_time,
                libvlc_media_player_get_position: s_libvlc_media_player_get_position,
                libvlc_media_player_set_position: s_libvlc_media_player_set_position,
                libvlc_media_player_set_rate: s_libvlc_media_player_set_rate,
                libvlc_media_player_get_rate: s_libvlc_media_player_get_rate,
                libvlc_audio_get_volume: s_libvlc_audio_get_volume,
                libvlc_audio_set_volume: s_libvlc_audio_set_volume,
                libvlc_audio_get_mute: s_libvlc_audio_get_mute,
                libvlc_audio_set_mute: s_libvlc_audio_set_mute,
                libvlc_video_set_callbacks: s_libvlc_video_set_callbacks,
                libvlc_video_set_format_callbacks: s_libvlc_video_set_format_callbacks,
                libvlc_video_get_size: s_libvlc_video_get_size,
                libvlc_video_get_spu_count: s_libvlc_video_get_spu_count,
                libvlc_video_get_spu_description: s_libvlc_video_get_spu_description,
                libvlc_video_get_spu: s_libvlc_video_get_spu,
                libvlc_video_set_spu: s_libvlc_video_set_spu,
                libvlc_video_set_subtitle_file: s_libvlc_video_set_subtitle_file,
                libvlc_audio_get_track_count: s_libvlc_audio_get_track_count,
                libvlc_audio_get_track_description: s_libvlc_audio_get_track_description,
                libvlc_audio_get_track: s_libvlc_audio_get_track,
                libvlc_audio_set_track: s_libvlc_audio_set_track,
                libvlc_track_description_list_release: s_libvlc_track_description_list_release,
                libvlc_video_take_snapshot: s_libvlc_video_take_snapshot,
            };

            Ok(Arc::new(syms))
        }
    }

    pub fn version_string(&self) -> String {
        unsafe {
            let ptr = (self.libvlc_get_version)();
            if ptr.is_null() {
                "Unknown".to_string()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().to_string()
            }
        }
    }
}
