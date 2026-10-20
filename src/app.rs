use crate::audio::backend::{AudioBackend, AudioCommand, PlaybackState};
use crate::audio::metadata::TrackMetadata;
use crate::audio::visualizer::{VisualizerProcessor, NUM_SPECTRUM_BARS};
use crate::lyrics::Lyrics;
use crate::playlist::Playlist;
use crate::ui::ab_loop::{render_tools_view, AbLoopState, SleepTimerState};
use crate::ui::equalizer_view::{render_equalizer_view, EqualizerUiState};
use crate::ui::inspector_view::render_inspector_view;
use crate::ui::now_playing::{render_now_playing, NowPlayingUiState};
use crate::ui::player_bar::render_player_bar;
use crate::ui::playlist_view::render_playlist_view;
use crate::ui::radio_view::{render_radio_view, RadioUiState};
use crate::ui::video_view::{render_video_view, VideoUiState};
use crate::utils::config::AppConfig;
use crate::video::VideoBackend;
use egui::{Color32, ColorImage, Key, RichText, Stroke, TextureHandle};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppTab {
    NowPlaying,
    Video,
    Equalizer,
    Playlist,
    Radio,
    Inspector,
    Tools,
}

impl AppTab {
    pub fn label(&self) -> &'static str {
        match self {
            AppTab::NowPlaying => "🎵 Now Playing",
            AppTab::Video => "🎬 Video",
            AppTab::Equalizer => "🎚 Equalizer",
            AppTab::Playlist => "📑 Playlist",
            AppTab::Radio => "📻 Radio Streams",
            AppTab::Inspector => "ℹ️ Media Info",
            AppTab::Tools => "🛠 Tools & Loop",
        }
    }
}

pub struct XorinApp {
    audio: Option<AudioBackend>,
    video: Option<VideoBackend>,
    playlist: Playlist,
    current_tab: AppTab,
    playback_state: PlaybackState,
    current_position: f64,
    current_duration: f64,
    is_seekable: bool,
    volume: f32,
    pan: f32,
    speed: f32,
    muted: bool,
    active_metadata: Option<TrackMetadata>,
    active_lyrics: Option<Lyrics>,
    is_video_mode: bool,
    logo_texture: Option<TextureHandle>,
    // Sub-UI states
    now_playing_state: NowPlayingUiState,
    video_state: VideoUiState,
    equalizer_state: EqualizerUiState,
    radio_state: RadioUiState,
    ab_loop_state: AbLoopState,
    sleep_timer_state: SleepTimerState,
    // DSP & Visualizer
    vis_processor: VisualizerProcessor,
    spectrum_bars: [f32; NUM_SPECTRUM_BARS],
    peak_bars: [f32; NUM_SPECTRUM_BARS],
    config: AppConfig,
    error_message: Option<String>,
}

impl XorinApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set modern shadcn-style dark visual styling
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(Color32::from_rgb(250, 250, 250)); // shadcn foreground
        visuals.panel_fill = Color32::from_rgb(10, 10, 12); // shadcn background (clean obsidian black)
        visuals.window_fill = Color32::from_rgb(18, 18, 22); // shadcn popover / card
        visuals.extreme_bg_color = Color32::from_rgb(14, 14, 18);
        visuals.faint_bg_color = Color32::from_rgb(24, 24, 28);

        // Window & Menu / Popover (Shadcn Popover / Dropdown)
        visuals.window_stroke = Stroke::new(1.0_f32, Color32::from_rgb(39, 39, 45)); // border
        visuals.window_corner_radius = egui::CornerRadius::same(6);
        visuals.menu_corner_radius = egui::CornerRadius::same(6);
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_rgba_premultiplied(0, 0, 0, 180),
        };

        // Selection / Accent (Clean Orange brand accent preserved)
        visuals.selection.bg_fill = Color32::from_rgb(249, 115, 22); // primary (amber/orange)
        visuals.selection.stroke = Stroke::new(1.0_f32, Color32::from_rgb(251, 146, 60));

        // Widgets: Inactive (shadcn secondary / outline)
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(18, 18, 22);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(18, 18, 22);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(39, 39, 45));
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(220, 220, 225));

        // Widgets: Hovered (shadcn hover state)
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(28, 28, 34);
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(28, 28, 34);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(55, 55, 65));
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(255, 255, 255));

        // Widgets: Active / Clicked
        visuals.widgets.active.bg_fill = Color32::from_rgb(34, 34, 42);
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(34, 34, 42);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(249, 115, 22));
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(255, 255, 255));

        // Widgets: Noninteractive
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(14, 14, 18);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(28, 28, 34));
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(161, 161, 170)); // muted-foreground

        // Make all clickable buttons/interactive elements show the pointing hand cursor
        visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

        cc.egui_ctx.set_visuals(visuals);

        // Shadcn UI Spacing & Interaction adjustments
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.combo_width = 110.0;
        style.interaction.show_tooltips_only_when_still = false;
        cc.egui_ctx.set_style(style);

        let config = AppConfig::load();
        let audio_backend = AudioBackend::new().ok();
        let video_backend = VideoBackend::new(None).ok();

        // Load dark-logo.png into TextureHandle
        let logo_texture = {
            let logo_bytes = include_bytes!("../dark-logo.png");
            if let Ok(img) = image::load_from_memory(logo_bytes) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let color_img = ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    rgba.as_flat_samples().as_slice(),
                );
                Some(cc.egui_ctx.load_texture("dark_logo", color_img, egui::TextureOptions::LINEAR))
            } else {
                None
            }
        };

        let app = Self {
            audio: audio_backend,
            video: video_backend,
            playlist: Playlist::new(),
            current_tab: AppTab::NowPlaying,
            playback_state: PlaybackState::Stopped,
            current_position: 0.0,
            current_duration: 0.0,
            is_seekable: true,
            volume: config.volume,
            pan: config.pan,
            speed: config.speed,
            muted: false,
            active_metadata: None,
            active_lyrics: None,
            is_video_mode: false,
            logo_texture,
            now_playing_state: NowPlayingUiState::default(),
            video_state: VideoUiState::default(),
            equalizer_state: EqualizerUiState {
                enabled: config.eq_enabled,
                preamp: config.eq_preamp,
                bands: config.eq_bands,
                selected_preset: config.eq_preset_name.clone(),
            },
            radio_state: RadioUiState::default(),
            ab_loop_state: AbLoopState::default(),
            sleep_timer_state: SleepTimerState::default(),
            vis_processor: VisualizerProcessor::new(),
            spectrum_bars: [0.0; NUM_SPECTRUM_BARS],
            peak_bars: [0.0; NUM_SPECTRUM_BARS],
            config,
            error_message: None,
        };

        // Sync initial state to audio backend
        if let Some(ref backend) = app.audio {
            backend.send_command(AudioCommand::SetVolume(app.volume));
            backend.send_command(AudioCommand::SetPan(app.pan));
            backend.send_command(AudioCommand::SetSpeed(app.speed));
            backend.send_command(AudioCommand::SetEqEnabled(app.equalizer_state.enabled));
            backend.send_command(AudioCommand::SetEqPreamp(app.equalizer_state.preamp));
            backend.send_command(AudioCommand::SetEqBands(app.equalizer_state.bands));
        }

        app
    }

    pub fn play_current_track(&mut self) {
        if let Some(track) = self.playlist.current_track().cloned() {
            self.start_playback(&track.path_or_url, track.is_stream, Some(track.title.clone()));
        }
    }

    pub fn start_playback(&mut self, path_or_url: &str, is_stream: bool, _default_title: Option<String>) {
        let is_video = !is_stream && crate::playlist::is_supported_video_extension(std::path::Path::new(path_or_url));
        self.is_video_mode = is_video;
        self.current_position = 0.0;
        self.error_message = None;

        if is_video {
            // Stop audio backend
            if let Some(ref audio) = self.audio {
                audio.send_command(AudioCommand::Stop);
            }

            // Start VLC video backend
            if let Some(ref vb) = self.video {
                if let Err(e) = vb.play(path_or_url) {
                    self.error_message = Some(format!("VLC Video Playback Error: {}", e));
                } else {
                    vb.set_volume(self.volume);
                    vb.set_mute(self.muted);
                    vb.set_speed(self.speed);
                    self.current_tab = AppTab::Video;
                    self.playback_state = PlaybackState::Playing;
                }
            } else {
                self.error_message = Some("VLC Engine not available. Please install VLC Media Player.".to_string());
                self.current_tab = AppTab::Video;
            }

            let path = PathBuf::from(path_or_url);
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("VIDEO").to_uppercase();
            if let Some(track) = self.playlist.current_track_mut() {
                track.codec = Some(format!("{} (VLC)", ext));
            }
            self.active_metadata = None;
            self.active_lyrics = None;
        } else {
            // Stop video backend if it was playing
            if let Some(ref vb) = self.video {
                vb.stop();
            }

            if let Some(ref backend) = self.audio {
                backend.send_command(AudioCommand::Play(path_or_url.to_string(), is_stream));
            }

            if !is_stream {
                let path = PathBuf::from(path_or_url);
                // Load metadata & cover art
                if let Ok(meta) = TrackMetadata::read_from_file(&path) {
                    self.current_duration = meta.duration_secs;
                    if let Some(track) = self.playlist.current_track_mut() {
                        if let Some(ref title) = meta.title {
                            track.title = title.clone();
                        }
                        if let Some(ref artist) = meta.artist {
                            track.artist = artist.clone();
                        }
                        if let Some(ref album) = meta.album {
                            track.album = album.clone();
                        }
                        track.duration = meta.duration_secs;
                        track.codec = Some(meta.codec_name.clone());
                        track.sample_rate = meta.sample_rate;
                        track.channels = meta.channels;
                        track.bitrate = meta.bitrate_kbps;
                    }
                    self.active_metadata = Some(meta);
                } else {
                    self.active_metadata = None;
                }

                // Load LRC lyrics if available
                self.active_lyrics = Lyrics::load_for_audio_file(&path);
            } else {
                self.current_duration = 0.0;
                self.active_metadata = None;
                self.active_lyrics = None;
            }
        }
    }

    pub fn play_next(&mut self) {
        if let Some((_, track)) = self.playlist.next() {
            let path = track.path_or_url.clone();
            let is_stream = track.is_stream;
            let title = track.title.clone();
            self.start_playback(&path, is_stream, Some(title));
        } else {
            self.stop();
        }
    }

    pub fn play_previous(&mut self) {
        // If track played for > 3 seconds, restart track instead of jumping back
        if self.current_position > 3.0 {
            self.seek(0.0);
            return;
        }

        if let Some((_, track)) = self.playlist.previous() {
            let path = track.path_or_url.clone();
            let is_stream = track.is_stream;
            let title = track.title.clone();
            self.start_playback(&path, is_stream, Some(title));
        }
    }

    pub fn toggle_play_pause(&mut self) {
        if self.is_video_mode {
            if let Some(ref vb) = self.video {
                vb.toggle_play_pause();
                self.playback_state = vb.state();
            }
        } else {
            match self.playback_state {
                PlaybackState::Playing => {
                    if let Some(ref b) = self.audio {
                        b.send_command(AudioCommand::Pause);
                    }
                    self.playback_state = PlaybackState::Paused;
                }
                PlaybackState::Paused => {
                    if let Some(ref b) = self.audio {
                        b.send_command(AudioCommand::Resume);
                    }
                    self.playback_state = PlaybackState::Playing;
                }
                PlaybackState::Stopped => {
                    if self.playlist.current_track().is_some() {
                        self.play_current_track();
                    } else if !self.playlist.is_empty() {
                        if let Some(track) = self.playlist.play_index(0).cloned() {
                            self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn stop(&mut self) {
        if self.is_video_mode {
            if let Some(ref vb) = self.video {
                vb.stop();
            }
        }
        if let Some(ref b) = self.audio {
            b.send_command(AudioCommand::Stop);
        }
        self.playback_state = PlaybackState::Stopped;
        self.current_position = 0.0;
    }

    pub fn seek(&mut self, time_secs: f64) {
        let clamped = time_secs.clamp(0.0, self.current_duration.max(0.0));
        self.current_position = clamped;
        if self.is_video_mode {
            if let Some(ref vb) = self.video {
                vb.seek(clamped);
            }
        } else if let Some(ref b) = self.audio {
            b.send_command(AudioCommand::Seek(clamped));
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 2.0);
        if let Some(ref b) = self.audio {
            b.send_command(AudioCommand::SetVolume(self.volume));
        }
        if let Some(ref vb) = self.video {
            vb.set_volume(self.volume);
        }
    }

    pub fn set_speed(&mut self, spd: f32) {
        self.speed = spd.clamp(0.25, 8.0);
        if let Some(ref b) = self.audio {
            b.send_command(AudioCommand::SetSpeed(self.speed));
        }
        if let Some(ref vb) = self.video {
            vb.set_speed(self.speed);
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if let Some(ref b) = self.audio {
            b.send_command(AudioCommand::SetMute(self.muted));
        }
        if let Some(ref vb) = self.video {
            vb.set_mute(self.muted);
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Space: Play/Pause
            if i.key_pressed(Key::Space) {
                self.toggle_play_pause();
            }

            // S: Stop
            if i.key_pressed(Key::S) && !i.modifiers.command && !i.modifiers.ctrl {
                self.stop();
            }

            // Left / Right Arrow: Seek
            let seek_step = if i.modifiers.shift { 15.0 } else { 5.0 };
            if i.key_pressed(Key::ArrowLeft) {
                self.seek((self.current_position - seek_step).max(0.0));
            }
            if i.key_pressed(Key::ArrowRight) {
                self.seek((self.current_position + seek_step).min(self.current_duration));
            }

            // Up / Down Arrow: Volume
            if i.key_pressed(Key::ArrowUp) {
                self.set_volume(self.volume + 0.05);
            }
            if i.key_pressed(Key::ArrowDown) {
                self.set_volume(self.volume - 0.05);
            }

            // M: Mute
            if i.key_pressed(Key::M) {
                self.toggle_mute();
            }

            // N: Next, P: Previous
            if i.key_pressed(Key::N) {
                self.play_next();
            }
            if i.key_pressed(Key::P) {
                self.play_previous();
            }

            // R: Repeat, Z: Shuffle
            if i.key_pressed(Key::R) {
                self.playlist.repeat_mode = self.playlist.repeat_mode.cycle();
            }
            if i.key_pressed(Key::Z) {
                self.playlist.toggle_shuffle();
            }

            // Number keys 1-7 to switch tabs
            if i.key_pressed(Key::Num1) {
                self.current_tab = AppTab::NowPlaying;
            }
            if i.key_pressed(Key::Num2) {
                self.current_tab = AppTab::Video;
            }
            if i.key_pressed(Key::Num3) {
                self.current_tab = AppTab::Equalizer;
            }
            if i.key_pressed(Key::Num4) {
                self.current_tab = AppTab::Playlist;
            }
            if i.key_pressed(Key::Num5) {
                self.current_tab = AppTab::Radio;
            }
            if i.key_pressed(Key::Num6) {
                self.current_tab = AppTab::Inspector;
            }
            if i.key_pressed(Key::Num7) {
                self.current_tab = AppTab::Tools;
            }

            // Escape key exits video fullscreen
            if i.key_pressed(Key::Escape) && self.video_state.is_fullscreen {
                self.video_state.is_fullscreen = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        });
    }

    fn handle_drag_and_drop(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            let was_empty = self.playlist.is_empty();
            let mut first_added_idx = None;

            for file in dropped {
                if let Some(path) = file.path {
                    if path.is_dir() {
                        self.playlist.add_directory(&path);
                    } else if path.is_file() {
                        let idx = self.playlist.len();
                        self.playlist.add_file(&path);
                        if first_added_idx.is_none() && self.playlist.len() > idx {
                            first_added_idx = Some(idx);
                        }
                    }
                }
            }

            if was_empty {
                if let Some(idx) = first_added_idx {
                    if let Some(track) = self.playlist.play_index(idx).cloned() {
                        self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                    }
                }
            }
        }
    }
}

impl eframe::App for XorinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll playback status
        if self.is_video_mode {
            if let Some(ref vb) = self.video {
                if vb.is_open() {
                    self.playback_state = vb.state();
                    self.current_position = vb.get_time();
                    let dur = vb.get_duration();
                    if dur > 0.0 {
                        self.current_duration = dur;
                        if let Some(track) = self.playlist.current_track_mut() {
                            if track.duration <= 0.0 {
                                track.duration = dur;
                            }
                        }
                    }
                    if vb.is_ended() {
                        self.play_next();
                    }
                }
            }
        } else if let Some(ref b) = self.audio {
            if let Some(status) = b.try_recv_status() {
                self.playback_state = status.state;
                self.current_position = status.position_secs;
                self.is_seekable = status.is_seekable;

                if let Some(err) = status.error {
                    self.error_message = Some(err);
                }

                if status.track_finished {
                    self.play_next();
                }
            }
        }

        // Sleep timer check
        if self.sleep_timer_state.is_expired() {
            self.stop();
            self.sleep_timer_state.stop();
        }

        // Handle user input
        self.handle_keyboard_shortcuts(ctx);
        self.handle_drag_and_drop(ctx);

        // Fetch live audio samples for visualizer
        let raw_samples = if let Some(ref b) = self.audio {
            b.visualizer_buf.get_samples()
        } else {
            vec![0.0; 1024]
        };

        let is_playing = self.playback_state == PlaybackState::Playing;
        let (bars, peaks) = self.vis_processor.process_spectrum(&raw_samples, 0.88);
        self.spectrum_bars = bars;
        self.peak_bars = peaks;

        let is_fullscreen_video = self.video_state.is_fullscreen && self.current_tab == AppTab::Video;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(is_fullscreen_video));

        // 1. Top Header & Navigation Tabs (hidden in fullscreen video)
        if !is_fullscreen_video {
            egui::TopBottomPanel::top("xorin_header")
                .frame(
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(10, 10, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(39, 39, 45)))
                        .inner_margin(egui::Margin::symmetric(16, 10)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Topbar Logo (using dark-logo.png)
                        if let Some(ref logo) = self.logo_texture {
                            ui.image((logo.id(), egui::vec2(100.0, 29.0)));
                        } else {
                            ui.label(
                                RichText::new("XORIN")
                                    .strong()
                                    .size(16.0)
                                    .color(Color32::from_rgb(250, 250, 250)),
                        );
                            ui.label(
                                RichText::new("PLAYER")
                                    .size(12.0)
                                    .color(Color32::from_rgb(249, 115, 22)),
                            );
                        }

                        ui.add_space(20.0);

                        // Navigation Tabs (Shadcn Tabs list style with orange active base)
                        let tabs = [
                            AppTab::NowPlaying,
                            AppTab::Video,
                            AppTab::Equalizer,
                            AppTab::Playlist,
                            AppTab::Radio,
                            AppTab::Inspector,
                            AppTab::Tools,
                        ];

                        for tab in tabs {
                            let is_active = self.current_tab == tab;
                            let text = if is_active {
                                RichText::new(tab.label())
                                    .strong()
                                    .color(Color32::WHITE)
                            } else {
                                RichText::new(tab.label())
                                    .color(Color32::from_rgb(161, 161, 170))
                            };

                            let (fill_color, border_stroke) = if is_active {
                                (
                                    Color32::from_rgb(249, 115, 22), // Base orange active background
                                    Stroke::new(1.0_f32, Color32::from_rgb(251, 146, 60)),
                                )
                            } else {
                                (Color32::TRANSPARENT, Stroke::NONE)
                            };

                            let tab_btn = egui::Button::new(text)
                                .fill(fill_color)
                                .stroke(border_stroke)
                                .corner_radius(egui::CornerRadius::same(6));

                            if ui.add(tab_btn).clicked() {
                                self.current_tab = tab;
                            }
                            ui.add_space(3.0);
                        }

                        // Status Indicator on the right (Shadcn Badge style)
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let status_text = match self.playback_state {
                                PlaybackState::Playing => "● Playing",
                                PlaybackState::Paused => "❚❚ Paused",
                                PlaybackState::Buffering => "◌ Buffering",
                                PlaybackState::Stopped => "■ Stopped",
                            };
                            let status_color = match self.playback_state {
                                PlaybackState::Playing => Color32::from_rgb(34, 197, 94),
                                PlaybackState::Paused => Color32::from_rgb(234, 179, 8),
                                PlaybackState::Buffering => Color32::from_rgb(59, 130, 246),
                                PlaybackState::Stopped => Color32::from_rgb(113, 113, 122),
                            };
                            ui.label(RichText::new(status_text).size(12.0).color(status_color));
                        });
                    });
                });

            // 2. Bottom Player Bar
            egui::TopBottomPanel::bottom("xorin_bottom_bar")
                .frame(
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(10, 10, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(39, 39, 45)))
                )
                .show(ctx, |ui| {
                    let action = render_player_bar(
                        ui,
                        self.playback_state,
                        self.current_position,
                        self.current_duration,
                        self.is_seekable,
                        self.volume,
                        self.muted,
                        self.speed,
                        self.playlist.shuffle_enabled,
                        self.playlist.repeat_mode,
                    );

                    if action.toggle_play {
                        self.toggle_play_pause();
                    }
                    if action.stop {
                        self.stop();
                    }
                    if action.next {
                        self.play_next();
                    }
                    if action.previous {
                        self.play_previous();
                    }
                    if let Some(delta) = action.seek_relative {
                        self.seek(self.current_position + delta);
                    }
                    if let Some(abs_pos) = action.seek_absolute {
                        self.seek(abs_pos);
                    }
                    if let Some(vol) = action.volume_changed {
                        self.set_volume(vol);
                    }
                    if let Some(spd) = action.speed_changed {
                        self.set_speed(spd);
                    }
                    if action.toggle_mute {
                        self.toggle_mute();
                    }
                    if action.toggle_shuffle {
                        self.playlist.toggle_shuffle();
                    }
                    if action.cycle_repeat {
                        self.playlist.repeat_mode = self.playlist.repeat_mode.cycle();
                    }
                });
        }

        // 3. Central Content Panel according to active Tab
        let panel_frame = if is_fullscreen_video {
            egui::Frame::NONE.fill(Color32::from_rgb(8, 9, 12)).inner_margin(0)
        } else {
            egui::Frame::central_panel(&ctx.style())
        };

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // Display error toast / banner if any
            let mut dismiss_err = false;
            if let Some(ref err) = self.error_message {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("⚠️ {}", err)).color(Color32::from_rgb(255, 90, 90)));
                    if ui.small_button("Dismiss").clicked() {
                        dismiss_err = true;
                    }
                });
                ui.separator();
            }
            if dismiss_err {
                self.error_message = None;
            }

            match self.current_tab {
                AppTab::NowPlaying => {
                    let mut seek_req = None;
                    let current_track = self.playlist.current_track();
                    render_now_playing(
                        ui,
                        &mut self.now_playing_state,
                        current_track,
                        self.active_metadata.as_ref(),
                        self.active_lyrics.as_ref(),
                        self.current_position,
                        &self.spectrum_bars,
                        &self.peak_bars,
                        &raw_samples,
                        is_playing,
                        &mut seek_req,
                    );
                    if let Some(target) = seek_req {
                        self.seek(target);
                    }
                }
                AppTab::Video => {
                    let current_track = self.playlist.current_track();
                    let action = render_video_view(
                        ui,
                        &mut self.video_state,
                        self.video.as_ref(),
                        current_track,
                        self.playback_state,
                        self.current_position,
                        self.current_duration,
                        self.volume,
                        self.muted,
                        self.speed,
                    );

                    if action.toggle_play {
                        self.toggle_play_pause();
                    }
                    if let Some(abs_pos) = action.seek_absolute {
                        self.seek(abs_pos);
                    }
                    if let Some(delta) = action.seek_relative {
                        self.seek(self.current_position + delta);
                    }
                    if let Some(vol) = action.volume_changed {
                        self.set_volume(vol);
                    }
                    if let Some(spd) = action.speed_changed {
                        self.set_speed(spd);
                    }
                    if action.toggle_mute {
                        self.toggle_mute();
                    }
                    if action.open_file {
                        if let Some(file) = rfd::FileDialog::new()
                            .add_filter(
                                "Video Files",
                                &[
                                    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "ts", "m2ts", "vob",
                                    "mpg", "3gp", "ogv",
                                ],
                            )
                            .pick_file()
                        {
                            let idx = self.playlist.len();
                            self.playlist.add_file(&file);
                            if let Some(track) = self.playlist.play_index(idx).cloned() {
                                self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                            }
                        }
                    }
                    if action.load_subtitle {
                        if let Some(file) = rfd::FileDialog::new()
                            .add_filter("Subtitle Files", &["srt", "sub", "ass", "vtt"])
                            .pick_file()
                        {
                            if let Some(ref vb) = self.video {
                                let ok = vb.add_subtitle_file(&file);
                                let now = ui.input(|i| i.time);
                                let fname = file.file_name().and_then(|n| n.to_str()).unwrap_or("subtitle");
                                if ok {
                                    self.video_state.snapshot_notification =
                                        Some((format!("Subtitle loaded: {}", fname), now));
                                } else {
                                    self.video_state.snapshot_notification =
                                        Some((format!("Failed to load: {}", fname), now));
                                }
                            }
                        }
                    }
                    if let Some(sub_id) = action.change_subtitle {
                        if let Some(ref vb) = self.video {
                            vb.set_subtitle(sub_id);
                            self.video_state.current_subtitle_id = sub_id;
                        }
                    }
                    if let Some(audio_id) = action.change_audio_track {
                        if let Some(ref vb) = self.video {
                            vb.set_audio_track(audio_id);
                            self.video_state.current_audio_track_id = audio_id;
                        }
                    }
                    if action.take_snapshot {
                        let pic_dir = dirs::picture_dir().unwrap_or_else(|| PathBuf::from("."));
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        let filename = format!("xorin_snapshot_{}.png", ts);
                        let save_path = pic_dir.join(&filename);
                        let now = ui.input(|i| i.time);

                        if let Some((w, h, ref bytes)) = self.video_state.cached_frame_bytes {
                            if let Ok(()) = image::save_buffer(
                                &save_path,
                                bytes,
                                w,
                                h,
                                image::ExtendedColorType::Rgba8,
                            ) {
                                self.video_state.snapshot_notification =
                                    Some((format!("Snapshot saved: {}", filename), now));
                            } else {
                                self.video_state.snapshot_notification =
                                    Some(("Failed to save snapshot image".to_string(), now));
                            }
                        }
                    }
                }
                AppTab::Equalizer => {
                    let mut changed = false;
                    render_equalizer_view(ui, &mut self.equalizer_state, &mut changed);
                    if changed {
                        if let Some(ref b) = self.audio {
                            b.send_command(AudioCommand::SetEqEnabled(self.equalizer_state.enabled));
                            b.send_command(AudioCommand::SetEqPreamp(self.equalizer_state.preamp));
                            b.send_command(AudioCommand::SetEqBands(self.equalizer_state.bands));
                        }
                    }
                }
                AppTab::Playlist => {
                    let action = render_playlist_view(ui, &mut self.playlist);
                    if let Some(idx) = action.play_index {
                        if let Some(track) = self.playlist.play_index(idx).cloned() {
                            self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                        }
                    }
                    if let Some(idx) = action.remove_index {
                        self.playlist.remove(idx);
                    }
                    if action.add_files {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter(
                                "All Media",
                                &[
                                    "mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "aiff", "mp4",
                                    "mkv", "avi", "mov", "webm", "flv", "wmv", "ts", "m2ts", "vob", "mpg",
                                    "3gp", "ogv",
                                ],
                            )
                            .add_filter(
                                "Video Files",
                                &[
                                    "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "ts", "m2ts", "vob",
                                    "mpg", "3gp", "ogv",
                                ],
                            )
                            .add_filter(
                                "Audio Files",
                                &["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "aiff"],
                            )
                            .pick_files()
                        {
                            let was_empty = self.playlist.is_empty();
                            let start_idx = self.playlist.len();
                            for f in files {
                                self.playlist.add_file(&f);
                            }
                            if was_empty && self.playlist.len() > start_idx {
                                if let Some(track) = self.playlist.play_index(0).cloned() {
                                    self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                                }
                            }
                        }
                    }
                    if action.add_folder {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            let was_empty = self.playlist.is_empty();
                            let start_idx = self.playlist.len();
                            self.playlist.add_directory(&folder);
                            if was_empty && self.playlist.len() > start_idx {
                                if let Some(track) = self.playlist.play_index(0).cloned() {
                                    self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                                }
                            }
                        }
                    }
                    if action.import_m3u {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Playlists", &["m3u", "m3u8"])
                            .pick_file()
                        {
                            let _ = self.playlist.import_m3u(&path);
                        }
                    }
                    if action.export_m3u {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Playlists", &["m3u", "m3u8"])
                            .set_file_name("playlist.m3u")
                            .save_file()
                        {
                            let _ = self.playlist.export_m3u(&path);
                        }
                    }
                    if action.clear {
                        self.playlist.clear();
                        self.stop();
                    }
                }
                AppTab::Radio => {
                    let action = render_radio_view(ui, &mut self.radio_state);
                    if let Some((url, name)) = action.play_stream {
                        self.playlist.add_stream(&url, Some(&name));
                        let last_idx = self.playlist.len() - 1;
                        if let Some(track) = self.playlist.play_index(last_idx).cloned() {
                            self.start_playback(&track.path_or_url, track.is_stream, Some(track.title));
                        }
                    }
                    if let Some((url, name)) = action.add_to_playlist {
                        self.playlist.add_stream(&url, Some(&name));
                    }
                }
                AppTab::Inspector => {
                    let video_dims = if self.is_video_mode {
                        self.video.as_ref().and_then(|v| {
                            let d = v.dimensions();
                            if d.0 > 0 {
                                Some(d)
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    };
                    render_inspector_view(
                        ui,
                        self.playlist.current_track(),
                        self.active_metadata.as_ref(),
                        video_dims,
                    );
                }
                AppTab::Tools => {
                    let mut ab_changed = false;
                    render_tools_view(
                        ui,
                        &mut self.ab_loop_state,
                        &mut self.sleep_timer_state,
                        self.current_position,
                        &mut ab_changed,
                    );
                    if ab_changed {
                        if let Some(ref b) = self.audio {
                            let ab = if self.ab_loop_state.is_looping {
                                match (self.ab_loop_state.point_a, self.ab_loop_state.point_b) {
                                    (Some(a), Some(b)) => Some((a, b)),
                                    _ => None,
                                }
                            } else {
                                None
                            };
                            b.send_command(AudioCommand::SetAbLoop(ab));
                        }
                    }
                }
            }
        });

        // Request repaint at 60 FPS while playing or in video mode for smooth playback
        if is_playing || self.is_video_mode || self.spectrum_bars.iter().any(|&b| b > 0.01) {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save user preferences
        self.config.volume = self.volume;
        self.config.pan = self.pan;
        self.config.speed = self.speed;
        self.config.eq_enabled = self.equalizer_state.enabled;
        self.config.eq_preamp = self.equalizer_state.preamp;
        self.config.eq_bands = self.equalizer_state.bands;
        self.config.eq_preset_name = self.equalizer_state.selected_preset.clone();
        self.config.save();
    }
}
