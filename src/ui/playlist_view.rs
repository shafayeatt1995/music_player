use crate::playlist::Playlist;
use crate::utils::time::format_time;
use egui::{Color32, RichText, Stroke, Ui, Vec2};

pub struct PlaylistAction {
    pub play_index: Option<usize>,
    pub remove_index: Option<usize>,
    pub add_files: bool,
    pub add_folder: bool,
    pub import_m3u: bool,
    pub export_m3u: bool,
    pub clear: bool,
}

impl Default for PlaylistAction {
    fn default() -> Self {
        Self {
            play_index: None,
            remove_index: None,
            add_files: false,
            add_folder: false,
            import_m3u: false,
            export_m3u: false,
            clear: false,
        }
    }
}

pub fn render_playlist_view(
    ui: &mut Ui,
    playlist: &mut Playlist,
) -> PlaylistAction {
    let mut action = PlaylistAction::default();

    ui.vertical(|ui| {
        // Top Toolbar: Search + Add buttons + Import/Export
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍 Search:").size(13.0));
            ui.add_sized(
                Vec2::new(180.0, 24.0),
                egui::TextEdit::singleline(&mut playlist.search_query).hint_text("Filter tracks..."),
            );

            if !playlist.search_query.is_empty() && ui.button("✖").clicked() {
                playlist.search_query.clear();
            }

            ui.add_space(15.0);

            if ui.button("➕ Add Media Files").on_hover_text("Open media files (MP4, MKV, AVI, MOV, WebM, MP3, FLAC, WAV, AAC, etc.)").clicked() {
                action.add_files = true;
            }

            if ui.button("📁 Add Folder").on_hover_text("Scan folder recursively for music").clicked() {
                action.add_folder = true;
            }

            if ui.button("📥 Import M3U").on_hover_text("Load M3U or M3U8 playlist file").clicked() {
                action.import_m3u = true;
            }

            if ui.button("💾 Export M3U").on_hover_text("Save current playlist to M3U").clicked() {
                action.export_m3u = true;
            }

            if !playlist.is_empty() && ui.button(RichText::new("🗑 Clear").color(Color32::from_rgb(255, 100, 100))).clicked() {
                action.clear = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let count = playlist.len();
                let total_secs = playlist.total_duration();
                ui.label(
                    RichText::new(format!("{} tracks ({})", count, format_time(total_secs)))
                        .small()
                        .color(Color32::GRAY),
                );
            });
        });

        ui.add_space(8.0);

        // Track table / list
        let filtered = playlist.filtered_indices();

        if playlist.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(RichText::new("Playlist is empty").size(16.0).color(Color32::GRAY));
                ui.label(RichText::new("Drag and drop audio files here, or use '+ Add Files' / '+ Add Folder' above").small());
                ui.add_space(40.0);
            });
        } else {
            // Table Header
            egui::Frame::canvas(ui.style())
                .fill(Color32::from_rgb(22, 25, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 45, 58)))
                .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("#").strong().size(12.0));
                    ui.add_space(18.0);
                    ui.label(RichText::new("Title").strong().size(12.0));
                    ui.add_space(ui.available_width() - 320.0);
                    ui.label(RichText::new("Artist").strong().size(12.0));
                    ui.add_space(ui.available_width() - 120.0);
                    ui.label(RichText::new("Duration").strong().size(12.0));
                });
            });

        ui.add_space(4.0);

        let current_playing_idx = playlist.current_index();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for &idx in &filtered {
                    let is_active = current_playing_idx == Some(idx);
                    let track = &playlist.tracks()[idx];

                    let bg_color = if is_active {
                        Color32::from_rgb(45, 35, 20)
                    } else if idx % 2 == 0 {
                        Color32::from_rgb(18, 20, 26)
                    } else {
                        Color32::from_rgb(22, 24, 30)
                    };

                    let row_stroke = if is_active {
                        Stroke::new(1.0_f32, Color32::from_rgb(255, 140, 20))
                    } else {
                        Stroke::NONE
                    };

                    egui::Frame::NONE
                        .fill(bg_color)
                        .stroke(row_stroke)
                        .inner_margin(egui::Margin::symmetric(10, 5))
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Index or Play icon
                                let num_str = if is_active {
                                    "▶".to_string()
                                } else {
                                    format!("{}", idx + 1)
                                };

                                let num_text = if is_active {
                                    RichText::new(num_str).strong().color(Color32::from_rgb(255, 160, 40))
                                } else {
                                    RichText::new(num_str).color(Color32::GRAY)
                                };

                                ui.label(num_text);
                                ui.add_space(8.0);

                                // Media type icon
                                let type_icon = if track.is_video() {
                                    "🎬"
                                } else if track.is_stream {
                                    "📻"
                                } else {
                                    "🎵"
                                };
                                ui.label(RichText::new(type_icon).size(12.0));
                                ui.add_space(4.0);

                                // Title (Clickable)
                                let title_color = if is_active {
                                    Color32::from_rgb(255, 180, 50)
                                } else {
                                    Color32::WHITE
                                };

                                let title_label = ui.add(
                                    egui::Label::new(
                                        RichText::new(&track.title)
                                            .strong()
                                            .color(title_color),
                                    )
                                    .sense(egui::Sense::click()),
                                );

                                if title_label.double_clicked() || title_label.clicked() {
                                    action.play_index = Some(idx);
                                }

                                ui.add_space(ui.available_width() - 320.0);

                                // Artist
                                ui.label(
                                    RichText::new(&track.artist)
                                        .color(Color32::from_rgb(170, 175, 190)),
                                );

                                ui.add_space(ui.available_width() - 110.0);

                                // Duration
                                let dur_str = if track.duration > 0.0 {
                                    format_time(track.duration)
                                } else if track.is_stream {
                                    "LIVE".to_string()
                                } else {
                                    "--:--".to_string()
                                };
                                ui.label(
                                    RichText::new(dur_str)
                                        .monospace()
                                        .color(Color32::LIGHT_GRAY),
                                );

                                ui.add_space(10.0);

                                // Delete button
                                if ui.small_button("✖").clicked() {
                                    action.remove_index = Some(idx);
                                }
                            });
                        });

                    ui.add_space(2.0);
                }
            });
        }
    });

    action
}
