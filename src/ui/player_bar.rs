use crate::audio::PlaybackState;
use crate::playlist::RepeatMode;
use crate::utils::time::format_time;
use egui::{Color32, RichText, Slider, Stroke, Ui, Vec2};

pub struct PlayerBarAction {
    pub toggle_play: bool,
    pub stop: bool,
    pub next: bool,
    pub previous: bool,
    pub seek_relative: Option<f64>,
    pub seek_absolute: Option<f64>,
    pub volume_changed: Option<f32>,
    pub speed_changed: Option<f32>,
    pub toggle_mute: bool,
    pub toggle_shuffle: bool,
    pub cycle_repeat: bool,
}

impl Default for PlayerBarAction {
    fn default() -> Self {
        Self {
            toggle_play: false,
            stop: false,
            next: false,
            previous: false,
            seek_relative: None,
            seek_absolute: None,
            volume_changed: None,
            speed_changed: None,
            toggle_mute: false,
            toggle_shuffle: false,
            cycle_repeat: false,
        }
    }
}

pub fn render_player_bar(
    ui: &mut Ui,
    playback_state: PlaybackState,
    position: f64,
    duration: f64,
    is_seekable: bool,
    volume: f32,
    muted: bool,
    speed: f32,
    shuffle_enabled: bool,
    repeat_mode: RepeatMode,
) -> PlayerBarAction {
    let mut action = PlayerBarAction::default();

    egui::Frame::canvas(ui.style())
        .fill(Color32::from_rgb(18, 20, 26))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(35, 40, 52)))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            // 1. Scrub Slider & Time Row
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format_time(position))
                        .monospace()
                        .size(13.0)
                        .color(Color32::from_rgb(220, 220, 220)),
                );

                let max_duration = if duration > 0.0 { duration } else { 1.0 };
                let mut current_pos = position.clamp(0.0, max_duration);

                if is_seekable && duration > 0.0 {
                    let slider = Slider::new(&mut current_pos, 0.0..=max_duration)
                        .show_value(false)
                        .trailing_fill(true);

                    let slider_resp = ui.add_sized(
                        Vec2::new(ui.available_width() - 85.0, 16.0),
                        slider,
                    );

                    if slider_resp.drag_stopped() || (slider_resp.clicked() && slider_resp.changed()) {
                        action.seek_absolute = Some(current_pos);
                    }
                } else {
                    // Indeterminate / live stream bar
                    let progress = if duration > 0.0 {
                        (position / duration).clamp(0.0, 1.0) as f32
                    } else {
                        1.0
                    };
                    ui.add_sized(
                        Vec2::new(ui.available_width() - 85.0, 16.0),
                        egui::ProgressBar::new(progress).show_percentage(),
                    );
                }

                let time_right = if duration > 0.0 {
                    format!("-{}", format_time((duration - position).max(0.0)))
                } else {
                    "LIVE".to_string()
                };

                ui.label(
                    RichText::new(time_right)
                        .monospace()
                        .size(13.0)
                        .color(if duration > 0.0 {
                            Color32::from_rgb(160, 160, 160)
                        } else {
                            Color32::from_rgb(255, 80, 80)
                        }),
                );
            });

            ui.add_space(6.0);

            // 2. Playback Controls & Volume/Speed Row
            ui.horizontal(|ui| {
                // Left: Shuffle & Repeat
                let shuffle_color = if shuffle_enabled {
                    Color32::from_rgb(255, 160, 30)
                } else {
                    Color32::from_rgb(130, 130, 140)
                };
                if ui
                    .button(RichText::new("🔀").size(15.0).color(shuffle_color))
                    .on_hover_text("Shuffle (Z)")
                    .clicked()
                {
                    action.toggle_shuffle = true;
                }

                let repeat_label = match repeat_mode {
                    RepeatMode::Off => "🔁 Off",
                    RepeatMode::All => "🔁 All",
                    RepeatMode::Single => "🔂 1",
                };
                let repeat_color = match repeat_mode {
                    RepeatMode::Off => Color32::from_rgb(130, 130, 140),
                    _ => Color32::from_rgb(255, 160, 30),
                };
                if ui
                    .button(RichText::new(repeat_label).size(13.0).color(repeat_color))
                    .on_hover_text("Repeat mode (R)")
                    .clicked()
                {
                    action.cycle_repeat = true;
                }

                ui.add_space(20.0);

                // Center: Transport Buttons
                if ui.button(RichText::new("⏮").size(16.0)).on_hover_text("Previous (P)").clicked() {
                    action.previous = true;
                }

                if ui.button(RichText::new("⏪ 10s").size(12.0)).on_hover_text("Seek -10s").clicked() {
                    action.seek_relative = Some(-10.0);
                }

                // Play / Pause prominent button
                let (play_label, play_hover) = match playback_state {
                    PlaybackState::Playing => ("⏸ Pause", "Pause (Space)"),
                    PlaybackState::Buffering => ("⌛ Buffer", "Buffering..."),
                    _ => ("▶ Play", "Play (Space)"),
                };
                let play_btn = egui::Button::new(
                    RichText::new(play_label)
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(230, 110, 15));

                if ui.add_sized(Vec2::new(95.0, 30.0), play_btn).on_hover_text(play_hover).clicked() {
                    action.toggle_play = true;
                }

                if ui.button(RichText::new("⏹").size(16.0)).on_hover_text("Stop (S)").clicked() {
                    action.stop = true;
                }

                if ui.button(RichText::new("10s ⏩").size(12.0)).on_hover_text("Seek +10s").clicked() {
                    action.seek_relative = Some(10.0);
                }

                if ui.button(RichText::new("⏭").size(16.0)).on_hover_text("Next (N)").clicked() {
                    action.next = true;
                }

                ui.add_space(20.0);

                // Right: Speed & Volume Controls
                ui.label(RichText::new("Speed:").size(12.0));
                let mut current_speed = speed;
                let speed_slider = Slider::new(&mut current_speed, 0.25..=4.0)
                    .step_by(0.05)
                    .show_value(false);

                if ui.add_sized(Vec2::new(75.0, 16.0), speed_slider).changed() {
                    action.speed_changed = Some(current_speed);
                }

                if ui
                    .button(RichText::new(format!("{:.2}x", speed)).size(11.0))
                    .on_hover_text("Click to reset speed to 1.0x")
                    .clicked()
                {
                    action.speed_changed = Some(1.0);
                }

                ui.add_space(15.0);

                // Volume Controls (0% to 200%)
                let vol_icon = if muted || volume == 0.0 {
                    "🔇"
                } else if volume < 0.5 {
                    "🔈"
                } else if volume <= 1.0 {
                    "🔉"
                } else {
                    "🔊"
                };

                if ui.button(RichText::new(vol_icon).size(15.0)).on_hover_text("Toggle Mute (M)").clicked() {
                    action.toggle_mute = true;
                }

                let mut current_vol = volume;
                let vol_slider = Slider::new(&mut current_vol, 0.0..=2.0)
                    .show_value(false)
                    .trailing_fill(true);

                if ui.add_sized(Vec2::new(90.0, 16.0), vol_slider).changed() {
                    action.volume_changed = Some(current_vol);
                }

                let vol_pct = (volume * 100.0).round() as u32;
                let vol_color = if vol_pct > 100 {
                    Color32::from_rgb(255, 120, 40) // Amber for VLC boost > 100%
                } else {
                    Color32::LIGHT_GRAY
                };
                ui.label(
                    RichText::new(format!("{}%", vol_pct))
                        .size(12.0)
                        .monospace()
                        .color(vol_color),
                );
            });
        });

    action
}
