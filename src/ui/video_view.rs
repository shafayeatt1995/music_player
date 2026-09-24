use crate::audio::PlaybackState;
use crate::playlist::track::Track;
use crate::utils::time::format_time;
use crate::video::backend::VideoBackend;
use crate::video::TrackItem;
use egui::{Color32, CornerRadius, Key, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AspectRatioMode {
    Original,
    Ratio16_9,
    Ratio4_3,
    Ratio21_9,
    Fill,
}

impl AspectRatioMode {
    pub fn label(&self) -> &'static str {
        match self {
            AspectRatioMode::Original => "Original",
            AspectRatioMode::Ratio16_9 => "16:9",
            AspectRatioMode::Ratio4_3 => "4:3",
            AspectRatioMode::Ratio21_9 => "21:9",
            AspectRatioMode::Fill => "Fill Window",
        }
    }
}

pub struct VideoUiState {
    pub aspect_ratio_mode: AspectRatioMode,
    pub is_fullscreen: bool,
    pub texture: Option<egui::TextureHandle>,
    pub cached_frame_bytes: Option<(u32, u32, Vec<u8>)>,
    pub mouse_idle_secs: f64,
    pub snapshot_notification: Option<(String, f64)>,
    pub available_subtitles: Vec<TrackItem>,
    pub available_audio_tracks: Vec<TrackItem>,
    pub current_subtitle_id: i32,
    pub current_audio_track_id: i32,
    pub last_tracks_poll: f64,
}

impl Default for VideoUiState {
    fn default() -> Self {
        Self {
            aspect_ratio_mode: AspectRatioMode::Original,
            is_fullscreen: false,
            texture: None,
            cached_frame_bytes: None,
            mouse_idle_secs: 0.0,
            snapshot_notification: None,
            available_subtitles: Vec::new(),
            available_audio_tracks: Vec::new(),
            current_subtitle_id: -1,
            current_audio_track_id: -1,
            last_tracks_poll: 0.0,
        }
    }
}

pub struct VideoViewAction {
    pub toggle_play: bool,
    pub seek_absolute: Option<f64>,
    pub seek_relative: Option<f64>,
    pub open_file: bool,
    pub load_subtitle: bool,
    pub change_subtitle: Option<i32>,
    pub change_audio_track: Option<i32>,
    pub take_snapshot: bool,
    pub volume_changed: Option<f32>,
    pub speed_changed: Option<f32>,
    pub toggle_mute: bool,
}

impl Default for VideoViewAction {
    fn default() -> Self {
        Self {
            toggle_play: false,
            seek_absolute: None,
            seek_relative: None,
            open_file: false,
            load_subtitle: false,
            change_subtitle: None,
            change_audio_track: None,
            take_snapshot: false,
            volume_changed: None,
            speed_changed: None,
            toggle_mute: false,
        }
    }
}

pub fn render_video_view(
    ui: &mut Ui,
    state: &mut VideoUiState,
    video_backend: Option<&VideoBackend>,
    current_track: Option<&Track>,
    playback_state: PlaybackState,
    position: f64,
    duration: f64,
    volume: f32,
    muted: bool,
    speed: f32,
) -> VideoViewAction {
    let mut action = VideoViewAction::default();
    let now = ui.input(|i| i.time);

    // Poll newly decoded frame from backend
    if let Some(vb) = video_backend {
        if let Some((w, h, rgba)) = vb.poll_new_frame() {
            let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
            if let Some(ref mut tex) = state.texture {
                if tex.size() == [w as usize, h as usize] {
                    tex.set(color_image, egui::TextureOptions::LINEAR);
                } else {
                    *tex = ui.ctx().load_texture("vlc_video_frame", color_image, egui::TextureOptions::LINEAR);
                }
            } else {
                state.texture = Some(ui.ctx().load_texture("vlc_video_frame", color_image, egui::TextureOptions::LINEAR));
            }
            state.cached_frame_bytes = Some((w, h, rgba));
            ui.ctx().request_repaint();
        }

        // Periodically refresh subtitle and audio tracks
        if now - state.last_tracks_poll > 2.0 && vb.is_open() {
            state.available_subtitles = vb.get_subtitles();
            state.available_audio_tracks = vb.get_audio_tracks();
            state.current_subtitle_id = vb.get_current_subtitle();
            state.current_audio_track_id = vb.get_current_audio_track();
            state.last_tracks_poll = now;
        }
    }

    let is_video_active = video_backend.map_or(false, |vb| vb.is_open()) && state.texture.is_some();

    // Mouse movement activity detection
    let mouse_moved = ui.input(|i| i.pointer.delta().length() > 0.5);
    if mouse_moved {
        state.mouse_idle_secs = now;
    }
    let controls_visible = (now - state.mouse_idle_secs < 3.0) || playback_state != PlaybackState::Playing;

    let available_rect = ui.available_rect_before_wrap();

    if !is_video_active {
        // Empty state / Drop zone & Info
        render_empty_video_state(ui, video_backend, &mut action);
        return action;
    }

    let texture = state.texture.as_ref().unwrap();
    let [tex_w, tex_h] = texture.size();

    // Compute aspect ratio
    let target_aspect = match state.aspect_ratio_mode {
        AspectRatioMode::Original => {
            if tex_h > 0 {
                tex_w as f32 / tex_h as f32
            } else {
                16.0 / 9.0
            }
        }
        AspectRatioMode::Ratio16_9 => 16.0 / 9.0,
        AspectRatioMode::Ratio4_3 => 4.0 / 3.0,
        AspectRatioMode::Ratio21_9 => 21.0 / 9.0,
        AspectRatioMode::Fill => available_rect.width() / available_rect.height().max(1.0),
    };

    let (draw_w, draw_h) = if state.aspect_ratio_mode == AspectRatioMode::Fill {
        (available_rect.width(), available_rect.height())
    } else {
        let avail_w = available_rect.width();
        let avail_h = available_rect.height();
        if avail_w / avail_h > target_aspect {
            (avail_h * target_aspect, avail_h)
        } else {
            (avail_w, avail_w / target_aspect)
        }
    };

    let draw_x = available_rect.min.x + (available_rect.width() - draw_w) * 0.5;
    let draw_y = available_rect.min.y + (available_rect.height() - draw_h) * 0.5;
    let video_rect = Rect::from_min_size(Pos2::new(draw_x, draw_y), Vec2::new(draw_w, draw_h));

    // Paint background pillar/letterbox bars
    ui.painter().rect_filled(available_rect, CornerRadius::ZERO, Color32::from_rgb(8, 9, 12));

    // Paint video frame
    ui.painter().image(
        texture.id(),
        video_rect,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    // Interactive response over the video
    let resp = ui.allocate_rect(available_rect, Sense::click());
    if resp.double_clicked() {
        state.is_fullscreen = !state.is_fullscreen;
    } else if resp.clicked() {
        action.toggle_play = true;
    }

    // Keyboard shortcut for Fullscreen
    if ui.input(|i| i.key_pressed(Key::F)) {
        state.is_fullscreen = !state.is_fullscreen;
    }

    // Render Snapshot banner if active
    if let Some((ref msg, time_sent)) = state.snapshot_notification {
        if now - time_sent < 3.0 {
            let toast_rect = Rect::from_min_size(
                Pos2::new(available_rect.center().x - 140.0, available_rect.min.y + 20.0),
                Vec2::new(280.0, 36.0),
            );
            ui.painter().rect_filled(
                toast_rect,
                CornerRadius::same(6),
                Color32::from_rgba_premultiplied(20, 25, 35, 230),
            );
            ui.painter().rect_stroke(
                toast_rect,
                CornerRadius::same(6),
                Stroke::new(1.0_f32, Color32::from_rgb(80, 200, 120)),
                StrokeKind::Inside,
            );
            ui.painter().text(
                toast_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("📷 {}", msg),
                egui::FontId::proportional(13.0),
                Color32::from_rgb(220, 255, 220),
            );
        } else {
            state.snapshot_notification = None;
        }
    }

    // Floating overlay controls
    if controls_visible {
        render_overlay_controls(
            ui,
            available_rect,
            state,
            current_track,
            playback_state,
            position,
            duration,
            volume,
            muted,
            speed,
            tex_w as u32,
            tex_h as u32,
            &mut action,
        );
    }

    action
}

fn render_overlay_controls(
    ui: &mut Ui,
    bounds: Rect,
    state: &mut VideoUiState,
    current_track: Option<&Track>,
    playback_state: PlaybackState,
    position: f64,
    duration: f64,
    volume: f32,
    muted: bool,
    speed: f32,
    width: u32,
    height: u32,
    action: &mut VideoViewAction,
) {
    let overlay_height = 80.0;
    let overlay_rect = Rect::from_min_max(
        Pos2::new(bounds.min.x + 12.0, bounds.max.y - overlay_height - 12.0),
        Pos2::new(bounds.max.x - 12.0, bounds.max.y - 12.0),
    );

    // Draw background translucent blur panel
    ui.painter().rect_filled(
        overlay_rect,
        CornerRadius::same(8),
        Color32::from_rgba_premultiplied(14, 16, 22, 220),
    );
    ui.painter().rect_stroke(
        overlay_rect,
        CornerRadius::same(8),
        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(50, 55, 75, 180)),
        StrokeKind::Inside,
    );

    let mut ui_child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(overlay_rect.shrink(10.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    // 1. Scrubber Slider & Timing
    ui_child.horizontal(|ui| {
        ui.label(
            RichText::new(format_time(position))
                .monospace()
                .size(12.0)
                .color(Color32::from_rgb(220, 220, 220)),
        );

        let max_duration = if duration > 0.0 { duration } else { 1.0 };
        let mut cur_pos = position.clamp(0.0, max_duration);

        let slider = egui::Slider::new(&mut cur_pos, 0.0..=max_duration)
            .show_value(false)
            .trailing_fill(true);

        let resp = ui.add_sized(Vec2::new(ui.available_width() - 85.0, 14.0), slider);
        if resp.drag_stopped() || (resp.clicked() && resp.changed()) {
            action.seek_absolute = Some(cur_pos);
        }

        ui.label(
            RichText::new(format_time(duration))
                .monospace()
                .size(12.0)
                .color(Color32::from_rgb(160, 165, 180)),
        );
    });

    ui_child.add_space(4.0);

    // 2. Playback Buttons & Features Row
    ui_child.horizontal(|ui| {
        // Play/Pause
        let play_text = match playback_state {
            PlaybackState::Playing => "❚❚",
            _ => "▶",
        };
        if ui
            .button(RichText::new(play_text).strong().size(14.0))
            .on_hover_text("Play / Pause (Space)")
            .clicked()
        {
            action.toggle_play = true;
        }

        ui.add_space(6.0);

        // Track title and resolution info
        if let Some(track) = current_track {
            let label = format!("{} ({}x{})", track.title, width, height);
            ui.label(
                RichText::new(label)
                    .strong()
                    .size(12.0)
                    .color(Color32::from_rgb(255, 170, 40)),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Fullscreen toggle
            let fs_icon = if state.is_fullscreen { "⤓ Exit FS" } else { "⛶ Fullscreen" };
            if ui.button(fs_icon).on_hover_text("Toggle Fullscreen (F)").clicked() {
                state.is_fullscreen = !state.is_fullscreen;
            }

            ui.add_space(6.0);

            // Snapshot capture button
            if ui
                .button("📷 Snapshot")
                .on_hover_text("Capture current frame to PNG image")
                .clicked()
            {
                action.take_snapshot = true;
            }

            ui.add_space(6.0);

            // Aspect Ratio selector
            egui::ComboBox::from_id_salt("video_aspect_mode")
                .selected_text(state.aspect_ratio_mode.label())
                .width(90.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.aspect_ratio_mode, AspectRatioMode::Original, "Original");
                    ui.selectable_value(&mut state.aspect_ratio_mode, AspectRatioMode::Ratio16_9, "16:9");
                    ui.selectable_value(&mut state.aspect_ratio_mode, AspectRatioMode::Ratio4_3, "4:3");
                    ui.selectable_value(&mut state.aspect_ratio_mode, AspectRatioMode::Ratio21_9, "21:9");
                    ui.selectable_value(&mut state.aspect_ratio_mode, AspectRatioMode::Fill, "Fill Window");
                });

            ui.add_space(6.0);

            // Subtitle track selector
            let sub_label = if state.current_subtitle_id < 0 {
                "Subtitles: Off".to_string()
            } else if let Some(t) = state.available_subtitles.iter().find(|t| t.id == state.current_subtitle_id) {
                format!("Sub: {}", t.name)
            } else {
                "Subtitles".to_string()
            };

            egui::ComboBox::from_id_salt("video_subtitles")
                .selected_text(sub_label)
                .width(110.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(state.current_subtitle_id < 0, "Disable").clicked() {
                        action.change_subtitle = Some(-1);
                    }
                    for sub in &state.available_subtitles {
                        if ui.selectable_label(state.current_subtitle_id == sub.id, &sub.name).clicked() {
                            action.change_subtitle = Some(sub.id);
                        }
                    }
                    ui.separator();
                    if ui.button("📂 Load Subtitle File (.srt)...").clicked() {
                        action.load_subtitle = true;
                    }
                });

            ui.add_space(6.0);

            // Audio track switcher
            let audio_label = if let Some(t) = state.available_audio_tracks.iter().find(|t| t.id == state.current_audio_track_id) {
                format!("Audio: {}", t.name)
            } else {
                "Audio Track".to_string()
            };

            egui::ComboBox::from_id_salt("video_audio_tracks")
                .selected_text(audio_label)
                .width(110.0)
                .show_ui(ui, |ui| {
                    for at in &state.available_audio_tracks {
                        if ui.selectable_label(state.current_audio_track_id == at.id, &at.name).clicked() {
                            action.change_audio_track = Some(at.id);
                        }
                    }
                });

            ui.add_space(6.0);

            // Playback speed dropdown
            let speed_label = format!("{:.2}x", speed);
            egui::ComboBox::from_id_salt("video_speed")
                .selected_text(speed_label)
                .width(60.0)
                .show_ui(ui, |ui| {
                    for &spd in &[0.5, 0.75, 1.0, 1.25, 1.5, 2.0] {
                        if ui.selectable_label((speed - spd).abs() < 0.05, format!("{:.2}x", spd)).clicked() {
                            action.speed_changed = Some(spd);
                        }
                    }
                });

            ui.add_space(6.0);

            // Volume & Mute
            let vol_icon = if muted || volume == 0.0 { "🔇" } else { "🔊" };
            if ui.small_button(vol_icon).on_hover_text("Toggle Mute (M)").clicked() {
                action.toggle_mute = true;
            }

            let mut cur_vol = volume;
            let vol_slider = egui::Slider::new(&mut cur_vol, 0.0..=2.0)
                .show_value(false)
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0));
            let v_resp = ui.add_sized(Vec2::new(60.0, 16.0), vol_slider);
            if v_resp.changed() {
                action.volume_changed = Some(cur_vol);
            }
        });
    });
}

fn render_empty_video_state(
    ui: &mut Ui,
    video_backend: Option<&VideoBackend>,
    action: &mut VideoViewAction,
) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            ui.label(RichText::new("🎬").size(52.0));
            ui.add_space(8.0);
            ui.label(
                RichText::new("Universal VLC Video Engine")
                    .size(24.0)
                    .strong()
                    .color(Color32::from_rgb(255, 160, 30)),
            );

            ui.add_space(6.0);
            ui.label(
                RichText::new("Play high-definition videos with full hardware acceleration and native VLC decoder power.")
                    .size(14.0)
                    .color(Color32::from_rgb(170, 175, 190)),
            );

            ui.add_space(18.0);

            // Badges of supported video containers & codecs
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                let formats = [
                    "MP4", "MKV", "AVI", "MOV", "WebM", "FLV", "WMV", "TS", "M2TS", "VOB", "MPEG", "3GP", "OGV",
                    "H.264", "HEVC / H.265", "VP9", "AV1", "ProRes",
                ];
                for f in formats {
                    egui::Frame::NONE
                        .fill(Color32::from_rgb(25, 28, 38))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 52, 70)))
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .corner_radius(CornerRadius::same(4))
                        .show(ui, |ui| {
                            ui.label(RichText::new(f).size(11.0).color(Color32::from_rgb(210, 220, 240)));
                        });
                }
            });

            ui.add_space(24.0);

            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() * 0.5 - 120.0);
                if ui
                    .add_sized(
                        Vec2::new(240.0, 38.0),
                        egui::Button::new(
                            RichText::new("📂 Open Video File...")
                                .strong()
                                .size(15.0)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(235, 115, 20)),
                    )
                    .clicked()
                {
                    action.open_file = true;
                }
            });

            ui.add_space(14.0);
            ui.label(
                RichText::new("💡 Tip: Drag and drop video files or folders anywhere into the player window.")
                    .size(12.0)
                    .color(Color32::from_rgb(140, 145, 160)),
            );

            ui.add_space(20.0);

            // Backend Status Indicator
            if let Some(vb) = video_backend {
                let ver = vb.version();
                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 28, 20))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 110, 60)))
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .corner_radius(CornerRadius::same(6))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("✓ LibVLC Engine Active: {} (Hardware Accelerated)", ver))
                                .size(12.0)
                                .color(Color32::from_rgb(80, 220, 130)),
                        );
                    });
            } else {
                egui::Frame::NONE
                    .fill(Color32::from_rgb(32, 22, 18))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(130, 60, 40)))
                    .inner_margin(egui::Margin::symmetric(14, 8))
                    .corner_radius(CornerRadius::same(6))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("⚠️ LibVLC was not found on your system")
                                    .strong()
                                    .size(12.0)
                                    .color(Color32::from_rgb(255, 140, 90)),
                            );
                            ui.label(
                                RichText::new("Please install VLC Media Player from www.videolan.org to activate wide video format support.")
                                    .size(11.0)
                                    .color(Color32::from_rgb(210, 180, 170)),
                            );
                        });
                    });
            }
        });
    });
}
