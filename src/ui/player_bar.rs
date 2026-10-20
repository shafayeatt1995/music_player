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
        .fill(Color32::from_rgb(10, 10, 12))
        .stroke(Stroke::NONE)
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            // 1. Scrub Slider & Time Row
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format_time(position))
                        .monospace()
                        .size(13.0)
                        .color(Color32::from_rgb(220, 220, 220)),
                );

                let time_right = if duration > 0.0 {
                    format!("-{}", format_time((duration - position).max(0.0)))
                } else {
                    format_time(0.0)
                };

                let right_galley = ui.painter().layout_no_wrap(
                    time_right.clone(),
                    egui::FontId::monospace(13.0),
                    Color32::from_rgb(160, 160, 160),
                );
                let right_width = right_galley.size().x;
                let spacing = ui.spacing().item_spacing.x;
                let slider_width = (ui.available_width() - right_width - spacing).max(20.0);

                let max_duration = if duration > 0.0 { duration } else { 1.0 };
                let mut current_pos = position.clamp(0.0, max_duration);

                if is_seekable && duration > 0.0 {
                    let slider = Slider::new(&mut current_pos, 0.0..=max_duration)
                        .show_value(false)
                        .trailing_fill(true);

                    let slider_resp = ui.add_sized(
                        Vec2::new(slider_width, 16.0),
                        slider,
                    );

                    if slider_resp.drag_stopped() || (slider_resp.clicked() && slider_resp.changed()) {
                        action.seek_absolute = Some(current_pos);
                    }
                } else {
                    // Non-seekable / empty / stopped state: show 0% flat bar and disallow clicking
                    let progress = if duration > 0.0 {
                        (position / duration).clamp(0.0, 1.0) as f32
                    } else {
                        0.0
                    };
                    ui.add_enabled(
                        false,
                        egui::ProgressBar::new(progress).desired_width(slider_width)
                    );
                }

                ui.label(
                    RichText::new(time_right)
                        .monospace()
                        .size(13.0)
                        .color(Color32::from_rgb(160, 160, 160)),
                );
            });

            ui.add_space(6.0);

            // 2. Playback Controls & Volume/Speed Row
            ui.horizontal(|ui| {
                let btn_h = 30.0;

                // Left: Shuffle & Repeat
                let shuffle_color = if shuffle_enabled {
                    Color32::from_rgb(255, 160, 30)
                } else {
                    Color32::from_rgb(130, 130, 140)
                };
                let shuffle_btn = egui::Button::new(RichText::new("🔀").size(15.0).color(shuffle_color));
                if ui
                    .add_sized(Vec2::new(36.0, btn_h), shuffle_btn)
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
                let repeat_btn = egui::Button::new(RichText::new(repeat_label).size(13.0).color(repeat_color));
                if ui
                    .add_sized(Vec2::new(65.0, btn_h), repeat_btn)
                    .on_hover_text("Repeat mode (R)")
                    .clicked()
                {
                    action.cycle_repeat = true;
                }

                ui.add_space(20.0);

                // Center: Transport Buttons
                let prev_btn = egui::Button::new(RichText::new("⏮").size(15.0));
                if ui.add_sized(Vec2::new(36.0, btn_h), prev_btn).on_hover_text("Previous (P)").clicked() {
                    action.previous = true;
                }

                let seek_back_btn = egui::Button::new(RichText::new("⏪ 10s").size(12.0));
                if ui.add_sized(Vec2::new(56.0, btn_h), seek_back_btn).on_hover_text("Seek -10s").clicked() {
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

                if ui.add_sized(Vec2::new(95.0, btn_h), play_btn).on_hover_text(play_hover).clicked() {
                    action.toggle_play = true;
                }

                let stop_btn = egui::Button::new(RichText::new("⏹").size(15.0));
                if ui.add_sized(Vec2::new(36.0, btn_h), stop_btn).on_hover_text("Stop (S)").clicked() {
                    action.stop = true;
                }

                let seek_fwd_btn = egui::Button::new(RichText::new("10s ⏩").size(12.0));
                if ui.add_sized(Vec2::new(56.0, btn_h), seek_fwd_btn).on_hover_text("Seek +10s").clicked() {
                    action.seek_relative = Some(10.0);
                }

                let next_btn = egui::Button::new(RichText::new("⏭").size(15.0));
                if ui.add_sized(Vec2::new(36.0, btn_h), next_btn).on_hover_text("Next (N)").clicked() {
                    action.next = true;
                }

                ui.add_space(20.0);

                // Right: Speed & Volume Controls
                ui.label(RichText::new("Speed:").size(12.0));

                let minus_spd_btn = egui::Button::new(RichText::new("➖").size(11.0));
                if ui
                    .add_sized(Vec2::new(26.0, btn_h), minus_spd_btn)
                    .on_hover_text("Decrease speed (-0.10x)")
                    .clicked()
                {
                    let new_spd = ((speed - 0.10) * 100.0).round() / 100.0;
                    action.speed_changed = Some(new_spd.clamp(0.25, 8.0));
                }

                let mut current_speed = speed;
                let speed_slider = Slider::new(&mut current_speed, 0.25..=8.0)
                    .step_by(0.05)
                    .show_value(false)
                    .trailing_fill(true);

                if ui.add_sized(Vec2::new(75.0, 16.0), speed_slider).changed() {
                    let snapped = if (current_speed - 1.0).abs() < 0.03 {
                        1.0
                    } else {
                        (current_speed * 20.0).round() / 20.0
                    };
                    action.speed_changed = Some(snapped);
                }

                let plus_spd_btn = egui::Button::new(RichText::new("➕").size(11.0));
                if ui
                    .add_sized(Vec2::new(26.0, btn_h), plus_spd_btn)
                    .on_hover_text("Increase speed (+0.10x)")
                    .clicked()
                {
                    let new_spd = ((speed + 0.10) * 100.0).round() / 100.0;
                    action.speed_changed = Some(new_spd.clamp(0.25, 8.0));
                }

                let speed_btn = egui::Button::new(RichText::new(format!("{:.2}x", speed)).size(11.0));
                if ui
                    .add_sized(Vec2::new(42.0, btn_h), speed_btn)
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

                let mute_btn = egui::Button::new(RichText::new(vol_icon).size(15.0));
                if ui
                    .add_sized(Vec2::new(32.0, btn_h), mute_btn)
                    .on_hover_text("Toggle Mute (M)")
                    .clicked()
                {
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
