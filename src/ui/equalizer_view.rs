use crate::audio::dsp::{EQ_LABELS, EQ_PRESETS};
use egui::{Color32, RichText, Slider, Stroke, Ui, Vec2};

pub struct EqualizerUiState {
    pub enabled: bool,
    pub preamp: f32,
    pub bands: [f32; 10],
    pub selected_preset: String,
}

impl Default for EqualizerUiState {
    fn default() -> Self {
        Self {
            enabled: true,
            preamp: 0.0,
            bands: [0.0; 10],
            selected_preset: "Flat".to_string(),
        }
    }
}

pub fn render_equalizer_view(
    ui: &mut Ui,
    state: &mut EqualizerUiState,
    on_change: &mut bool,
) {
    ui.vertical(|ui| {
        // Top controls: Enabled checkbox & Presets selector & Reset
        ui.horizontal(|ui| {
            if ui.checkbox(&mut state.enabled, RichText::new("Enable Equalizer").strong()).changed() {
                *on_change = true;
            }

            ui.add_space(20.0);
            ui.label(RichText::new("Preset:").strong());

            egui::ComboBox::from_id_salt("eq_preset_select")
                .selected_text(&state.selected_preset)
                .show_ui(ui, |ui| {
                    for preset in EQ_PRESETS {
                        if ui
                            .selectable_label(state.selected_preset == preset.name, preset.name)
                            .clicked()
                        {
                            state.selected_preset = preset.name.to_string();
                            state.preamp = preset.preamp;
                            state.bands = preset.bands;
                            *on_change = true;
                        }
                    }
                });

            ui.add_space(15.0);
            if ui.button("⟲ Reset Flat").clicked() {
                state.selected_preset = "Flat".to_string();
                state.preamp = 0.0;
                state.bands = [0.0; 10];
                *on_change = true;
            }
        });

        ui.add_space(15.0);

        // Grid of vertical sliders (Preamp + 10 Bands)
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(22, 24, 30))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 45, 58)))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    // Pre-amp column
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Preamp").small().color(Color32::from_rgb(255, 165, 0)));
                        ui.add_space(4.0);

                        let preamp_slider = Slider::new(&mut state.preamp, -20.0..=20.0)
                            .vertical()
                            .show_value(false);

                        if ui.add_sized(Vec2::new(32.0, 160.0), preamp_slider).changed() {
                            state.selected_preset = "Custom".to_string();
                            *on_change = true;
                        }

                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(format!("{:+.1}dB", state.preamp))
                                .small()
                                .color(Color32::LIGHT_GRAY),
                        );
                    });

                    ui.separator();
                    ui.add_space(4.0);

                    // 10 EQ Frequency Bands
                    for i in 0..10 {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(EQ_LABELS[i])
                                    .small()
                                    .color(Color32::from_rgb(180, 200, 230)),
                            );
                            ui.add_space(4.0);

                            let band_slider = Slider::new(&mut state.bands[i], -20.0..=20.0)
                                .vertical()
                                .show_value(false);

                            if ui.add_sized(Vec2::new(32.0, 160.0), band_slider).changed() {
                                state.selected_preset = "Custom".to_string();
                                *on_change = true;
                            }

                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(format!("{:+.1}dB", state.bands[i]))
                                    .small()
                                    .color(Color32::LIGHT_GRAY),
                            );
                        });
                        if i < 9 {
                            ui.add_space(2.0);
                        }
                    }
                });
            });
    });
}
