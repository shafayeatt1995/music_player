use crate::audio::TrackMetadata;
use crate::playlist::track::Track;
use crate::utils::time::format_time;
use egui::{Color32, RichText, Stroke, Ui};

pub fn render_inspector_view(
    ui: &mut Ui,
    current_track: Option<&Track>,
    metadata: Option<&TrackMetadata>,
    video_info: Option<(u32, u32)>,
) {
    ui.vertical(|ui| {
        ui.label(RichText::new("ℹ️ Media Information & Codec Inspector").strong().size(15.0));
        ui.label(RichText::new("Detailed format parameters and metadata tags").small().color(Color32::GRAY));
        ui.add_space(10.0);

        if let Some(track) = current_track {
            // General Info Section
            egui::Frame::canvas(ui.style())
                .fill(Color32::from_rgb(20, 24, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Track Metadata").strong().color(Color32::from_rgb(255, 170, 45)));
                    ui.separator();

                    egui::Grid::new("inspector_metadata_grid")
                        .striped(true)
                        .spacing([20.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Title:");
                            ui.label(RichText::new(&track.title).strong().color(Color32::WHITE));
                            ui.end_row();

                            ui.label("Artist:");
                            ui.label(RichText::new(&track.artist).color(Color32::LIGHT_GRAY));
                            ui.end_row();

                            ui.label("Album:");
                            ui.label(RichText::new(&track.album).color(Color32::LIGHT_GRAY));
                            ui.end_row();

                            if let Some(meta) = metadata {
                                if let Some(year) = meta.year {
                                    ui.label("Year:");
                                    ui.label(format!("{}", year));
                                    ui.end_row();
                                }
                                if let Some(ref genre) = meta.genre {
                                    ui.label("Genre:");
                                    ui.label(genre);
                                    ui.end_row();
                                }
                                if let Some(track_no) = meta.track_number {
                                    ui.label("Track Number:");
                                    ui.label(format!("{}", track_no));
                                    ui.end_row();
                                }
                            }

                            ui.label("Location:");
                            ui.label(
                                RichText::new(&track.path_or_url)
                                    .small()
                                    .color(Color32::from_rgb(140, 180, 240)),
                            );
                            ui.end_row();
                        });
                });

            ui.add_space(10.0);

            // Audio Technical Codec Specifications Section
            egui::Frame::canvas(ui.style())
                .fill(Color32::from_rgb(20, 24, 32))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(38, 46, 62)))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Codec & Stream Specifications").strong().color(Color32::from_rgb(255, 170, 45)));
                    ui.separator();

                    egui::Grid::new("inspector_codec_grid")
                        .striped(true)
                        .spacing([20.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Codec:");
                            let codec = metadata
                                .map(|m| m.codec_name.as_str())
                                .unwrap_or_else(|| track.codec.as_deref().unwrap_or("Unknown"));
                            ui.label(RichText::new(codec).strong().color(Color32::from_rgb(100, 220, 160)));
                            ui.end_row();

                            ui.label("Type:");
                            let type_str = if track.is_video() {
                                "Video File (VLC Engine)"
                            } else if track.is_stream {
                                "Live Stream (Network)"
                            } else {
                                "Local Audio File"
                            };
                            ui.label(type_str);
                            ui.end_row();

                            if let Some((w, h)) = video_info {
                                ui.label("Video Resolution:");
                                ui.label(RichText::new(format!("{} x {} pixels", w, h)).strong().color(Color32::from_rgb(255, 180, 50)));
                                ui.end_row();
                            }

                            if let Some(meta) = metadata {
                                if let Some(sr) = meta.sample_rate {
                                    ui.label("Sample Rate:");
                                    ui.label(format!("{} Hz ({:.1} kHz)", sr, sr as f32 / 1000.0));
                                    ui.end_row();
                                }

                                if let Some(bits) = meta.bits_per_sample {
                                    ui.label("Bit Depth:");
                                    ui.label(format!("{}-bit", bits));
                                    ui.end_row();
                                }

                                if let Some(channels) = meta.channels {
                                    ui.label("Channels:");
                                    let ch_name = match channels {
                                        1 => "Mono (1 channel)",
                                        2 => "Stereo (2 channels)",
                                        6 => "5.1 Surround (6 channels)",
                                        _ => "Multi-channel",
                                    };
                                    ui.label(ch_name);
                                    ui.end_row();
                                }

                                if let Some(kbps) = meta.bitrate_kbps {
                                    ui.label("Bitrate:");
                                    ui.label(format!("{} kbps", kbps));
                                    ui.end_row();
                                }
                            }

                            ui.label("Duration:");
                            let dur_str = if track.duration > 0.0 {
                                format!("{} ({} seconds)", format_time(track.duration), track.duration.round())
                            } else {
                                "Continuous Live Stream".to_string()
                            };
                            ui.label(dur_str);
                            ui.end_row();
                        });
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("No track selected or playing").size(15.0).color(Color32::GRAY));
                ui.label(RichText::new("Play a track to view its codec and technical properties").small());
                ui.add_space(50.0);
            });
        }
    });
}
