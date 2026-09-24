use egui::{Color32, RichText, Stroke, Ui, Vec2};

pub struct RadioStation {
    pub name: &'static str,
    pub genre: &'static str,
    pub url: &'static str,
    pub description: &'static str,
}

pub const RADIO_PRESETS: &[RadioStation] = &[
    RadioStation {
        name: "SomaFM: Groove Salad",
        genre: "Chillout / Ambient",
        url: "http://ice1.somafm.com/groovesalad-128-mp3",
        description: "A nicely chilled plate of ambient/downtempo beats and grooves.",
    },
    RadioStation {
        name: "SomaFM: Drone Zone",
        genre: "Atmospheric Ambient",
        url: "http://ice1.somafm.com/dronezone-128-mp3",
        description: "Served best chilled, safe with most medications. Atmospheric ambient space music.",
    },
    RadioStation {
        name: "SomaFM: Indie Pop Rocks!",
        genre: "Indie Rock / Pop",
        url: "http://ice1.somafm.com/indiepop-128-mp3",
        description: "New and classic favorite indie pop tracks.",
    },
    RadioStation {
        name: "SomaFM: Secret Agent",
        genre: "Spy / Lounge / Surf",
        url: "http://ice1.somafm.com/secretagent-128-mp3",
        description: "The soundtrack for your stylish, mysterious life. A spy cocktail.",
    },
    RadioStation {
        name: "SomaFM: Deep Space One",
        genre: "Deep Ambient / Space",
        url: "http://ice1.somafm.com/deepspaceone-128-mp3",
        description: "Deep ambient electronic and experimental space music for exploring inner and outer space.",
    },
    RadioStation {
        name: "SomaFM: DEF CON Radio",
        genre: "Hacker / Cyber / Synth",
        url: "http://ice1.somafm.com/defcon-128-mp3",
        description: "Music for hacking. The soundtrack of the DEF CON conference.",
    },
    RadioStation {
        name: "SomaFM: Suburbs of Goa",
        genre: "Desi-Beats / World Chill",
        url: "http://ice1.somafm.com/suburbsofgoa-128-mp3",
        description: "Desi-influenced Asian world beats and beyond.",
    },
    RadioStation {
        name: "SomaFM: Vaporwaves",
        genre: "Vaporwave / Chillwave",
        url: "http://ice1.somafm.com/vaporwaves-128-mp3",
        description: "All vaporwave, all the time. Surreal, nostalgic electronic music.",
    },
];

pub struct RadioAction {
    pub play_stream: Option<(String, String)>, // (url, name)
    pub add_to_playlist: Option<(String, String)>,
}

impl Default for RadioAction {
    fn default() -> Self {
        Self {
            play_stream: None,
            add_to_playlist: None,
        }
    }
}

pub struct RadioUiState {
    pub custom_url: String,
    pub custom_name: String,
}

impl Default for RadioUiState {
    fn default() -> Self {
        Self {
            custom_url: String::new(),
            custom_name: String::new(),
        }
    }
}

pub fn render_radio_view(
    ui: &mut Ui,
    state: &mut RadioUiState,
) -> RadioAction {
    let mut action = RadioAction::default();

    ui.vertical(|ui| {
        // Custom URL input section
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_rgb(20, 24, 32))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 68)))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(RichText::new("📡 Custom Audio Stream / Radio URL").strong().size(14.0));
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Stream URL:");
                    ui.add_sized(
                        Vec2::new(320.0, 24.0),
                        egui::TextEdit::singleline(&mut state.custom_url)
                            .hint_text("https://example.com/stream.mp3"),
                    );

                    ui.add_space(10.0);
                    ui.label("Name:");
                    ui.add_sized(
                        Vec2::new(140.0, 24.0),
                        egui::TextEdit::singleline(&mut state.custom_name)
                            .hint_text("Station Name (optional)"),
                    );

                    ui.add_space(10.0);

                    let can_play = !state.custom_url.trim().is_empty();
                    if ui.add_enabled(can_play, egui::Button::new("▶ Play Now")).clicked() {
                        let name = if state.custom_name.trim().is_empty() {
                            "Custom Stream".to_string()
                        } else {
                            state.custom_name.trim().to_string()
                        };
                        action.play_stream = Some((state.custom_url.trim().to_string(), name));
                    }

                    if ui.add_enabled(can_play, egui::Button::new("➕ Add to Playlist")).clicked() {
                        let name = if state.custom_name.trim().is_empty() {
                            "Custom Stream".to_string()
                        } else {
                            state.custom_name.trim().to_string()
                        };
                        action.add_to_playlist = Some((state.custom_url.trim().to_string(), name));
                    }
                });
            });

        ui.add_space(12.0);

        // Curated Stations Header
        ui.label(RichText::new("📻 Curated Internet Radio Stations").strong().size(14.0));
        ui.add_space(6.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for station in RADIO_PRESETS {
                    egui::Frame::canvas(ui.style())
                        .fill(Color32::from_rgb(18, 20, 26))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 36, 48)))
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(station.name)
                                                .strong()
                                                .size(13.0)
                                                .color(Color32::WHITE),
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            RichText::new(format!("[{}]", station.genre))
                                                .small()
                                                .color(Color32::from_rgb(255, 160, 40)),
                                        );
                                    });

                                    ui.label(
                                        RichText::new(station.description)
                                            .small()
                                            .color(Color32::from_rgb(140, 145, 160)),
                                    );
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("➕ Queue").on_hover_text("Add station to playlist").clicked() {
                                        action.add_to_playlist = Some((station.url.to_string(), station.name.to_string()));
                                    }

                                    if ui
                                        .button(RichText::new("▶ Play").strong().color(Color32::from_rgb(255, 150, 30)))
                                        .clicked()
                                    {
                                        action.play_stream = Some((station.url.to_string(), station.name.to_string()));
                                    }
                                });
                            });
                        });

                    ui.add_space(4.0);
                }
            });
    });

    action
}
