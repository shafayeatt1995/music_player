use crate::utils::time::format_time;
use egui::{Color32, RichText, Stroke, Ui};
use std::time::{Duration, Instant};

pub struct AbLoopState {
    pub point_a: Option<f64>,
    pub point_b: Option<f64>,
    pub is_looping: bool,
}

impl Default for AbLoopState {
    fn default() -> Self {
        Self {
            point_a: None,
            point_b: None,
            is_looping: false,
        }
    }
}

pub struct SleepTimerState {
    pub enabled: bool,
    pub target_time: Option<Instant>,
    pub duration_mins: u32,
}

impl Default for SleepTimerState {
    fn default() -> Self {
        Self {
            enabled: false,
            target_time: None,
            duration_mins: 30,
        }
    }
}

impl SleepTimerState {
    pub fn start(&mut self, minutes: u32) {
        self.duration_mins = minutes;
        self.target_time = Some(Instant::now() + Duration::from_secs((minutes * 60) as u64));
        self.enabled = true;
    }

    pub fn stop(&mut self) {
        self.enabled = false;
        self.target_time = None;
    }

    pub fn is_expired(&self) -> bool {
        if self.enabled {
            if let Some(target) = self.target_time {
                return Instant::now() >= target;
            }
        }
        false
    }

    pub fn remaining_secs(&self) -> Option<u64> {
        if self.enabled {
            if let Some(target) = self.target_time {
                let now = Instant::now();
                if target > now {
                    return Some((target - now).as_secs());
                }
            }
        }
        None
    }
}

pub fn render_tools_view(
    ui: &mut Ui,
    ab_state: &mut AbLoopState,
    sleep_state: &mut SleepTimerState,
    current_pos: f64,
    on_ab_change: &mut bool,
) {
    ui.vertical(|ui| {
        // 1. A-B Looping Section
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(20, 24, 32))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 68)))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(RichText::new("🔁 VLC A-B Loop Section").strong().size(14.0));
                ui.label(
                    RichText::new("Loop a precise portion of the audio track repeatedly (great for musicians and practice)")
                        .small()
                        .color(Color32::GRAY),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    // Point A
                    let a_label = match ab_state.point_a {
                        Some(a) => format_time(a),
                        None => "--:--".to_string(),
                    };
                    if ui.button(format!("📍 Set Point A [{}]", a_label)).clicked() {
                        ab_state.point_a = Some(current_pos);
                        if let Some(b) = ab_state.point_b {
                            if b > current_pos {
                                ab_state.is_looping = true;
                            }
                        }
                        *on_ab_change = true;
                    }

                    ui.add_space(10.0);

                    // Point B
                    let b_label = match ab_state.point_b {
                        Some(b) => format_time(b),
                        None => "--:--".to_string(),
                    };
                    if ui.button(format!("📍 Set Point B [{}]", b_label)).clicked() {
                        ab_state.point_b = Some(current_pos);
                        if let Some(a) = ab_state.point_a {
                            if current_pos > a {
                                ab_state.is_looping = true;
                            }
                        }
                        *on_ab_change = true;
                    }

                    ui.add_space(10.0);

                    // Clear A-B Loop
                    if (ab_state.point_a.is_some() || ab_state.point_b.is_some())
                        && ui.button("✖ Clear Loop").clicked()
                    {
                        ab_state.point_a = None;
                        ab_state.point_b = None;
                        ab_state.is_looping = false;
                        *on_ab_change = true;
                    }

                    if ab_state.is_looping {
                        ui.label(
                            RichText::new("● A-B LOOP ACTIVE")
                                .strong()
                                .color(Color32::from_rgb(255, 150, 30)),
                        );
                    }
                });
            });

        ui.add_space(14.0);

        // 2. Sleep Timer Section
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(20, 24, 32))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 68)))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(RichText::new("⏱ Sleep Timer").strong().size(14.0));
                ui.label(
                    RichText::new("Automatically stops playback after designated duration")
                        .small()
                        .color(Color32::GRAY),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    for mins in [15, 30, 45, 60, 90] {
                        let is_sel = sleep_state.enabled && sleep_state.duration_mins == mins;
                        if ui.selectable_label(is_sel, format!("{} mins", mins)).clicked() {
                            sleep_state.start(mins);
                        }
                    }

                    ui.add_space(10.0);

                    if sleep_state.enabled {
                        if ui.button(RichText::new("Turn Off").color(Color32::from_rgb(255, 100, 100))).clicked() {
                            sleep_state.stop();
                        }

                        if let Some(rem) = sleep_state.remaining_secs() {
                            let m = rem / 60;
                            let s = rem % 60;
                            ui.label(
                                RichText::new(format!("⏳ Stopping in {:02}:{:02}", m, s))
                                    .strong()
                                    .color(Color32::from_rgb(255, 180, 40)),
                            );
                        }
                    } else {
                        ui.label(RichText::new("Sleep timer is off").small().color(Color32::GRAY));
                    }
                });
            });
    });
}
