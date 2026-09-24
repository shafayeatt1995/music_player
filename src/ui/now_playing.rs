use crate::audio::visualizer::NUM_SPECTRUM_BARS;
use crate::audio::TrackMetadata;
use crate::lyrics::Lyrics;
use crate::playlist::track::Track;
use crate::ui::visualizer_view::{render_visualizer, VisualizerStyle};
use egui::{Color32, ColorImage, Pos2, RichText, Stroke, TextureHandle, Ui, Vec2};

pub struct NowPlayingUiState {
    pub visualizer_style: VisualizerStyle,
    pub art_texture: Option<TextureHandle>,
    pub last_art_bytes: Option<Vec<u8>>,
    pub show_lyrics: bool,
}

impl Default for NowPlayingUiState {
    fn default() -> Self {
        Self {
            visualizer_style: VisualizerStyle::SpectrumBars,
            art_texture: None,
            last_art_bytes: None,
            show_lyrics: true,
        }
    }
}

pub fn render_now_playing(
    ui: &mut Ui,
    state: &mut NowPlayingUiState,
    current_track: Option<&Track>,
    metadata: Option<&TrackMetadata>,
    lyrics: Option<&Lyrics>,
    current_pos: f64,
    bars: &[f32; NUM_SPECTRUM_BARS],
    peaks: &[f32; NUM_SPECTRUM_BARS],
    raw_samples: &[f32],
    is_playing: bool,
    seek_request: &mut Option<f64>,
) {
    // 1. Update Album Art texture if track changed
    update_album_art(ui.ctx(), state, metadata);

    ui.horizontal(|ui| {
        // Left Column: Album Art + Track Info + Format Badge
        ui.vertical(|ui| {
            ui.set_width(260.0);

            // Album Artwork Frame
            let art_size = Vec2::new(240.0, 240.0);
            if let Some(ref tex) = state.art_texture {
                ui.image((tex.id(), art_size));
            } else {
                render_fallback_artwork(ui, art_size, current_track.is_some());
            }

            ui.add_space(10.0);

            // Track details
            if let Some(track) = current_track {
                ui.label(
                    RichText::new(&track.title)
                        .size(17.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(&track.artist)
                        .size(14.0)
                        .color(Color32::from_rgb(200, 200, 210)),
                );
                ui.label(
                    RichText::new(&track.album)
                        .size(12.0)
                        .color(Color32::from_rgb(140, 140, 150)),
                );

                ui.add_space(6.0);

                // VLC-style Format / Codec Badge
                let badge_text = if track.is_stream {
                    "🔴 LIVE STREAM".to_string()
                } else if let Some(meta) = metadata {
                    let mut parts = Vec::new();
                    if !meta.codec_name.is_empty() {
                        parts.push(meta.codec_name.clone());
                    }
                    if let Some(sr) = meta.sample_rate {
                        parts.push(format!("{:.1} kHz", sr as f32 / 1000.0));
                    }
                    if let Some(bits) = meta.bits_per_sample {
                        parts.push(format!("{}-bit", bits));
                    }
                    if let Some(kbps) = meta.bitrate_kbps {
                        parts.push(format!("{} kbps", kbps));
                    }
                    parts.join(" • ")
                } else {
                    track.codec.clone().unwrap_or_else(|| "Audio Track".to_string())
                };

                egui::Frame::canvas(ui.style())
                    .fill(Color32::from_rgb(28, 32, 42))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgb(45, 55, 75)))
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(badge_text)
                                .size(11.0)
                                .monospace()
                                .color(Color32::from_rgb(255, 170, 50)),
                        );
                    });
            } else {
                ui.label(RichText::new("No Track Playing").size(15.0).italics());
                ui.label(RichText::new("Drag & drop files or add from playlist").small());
            }
        });

        ui.separator();
        ui.add_space(8.0);

        // Right Column: Real-time Visualizer & Synced Lyrics
        ui.vertical(|ui| {
            // Visualizer Header & Style Switcher
            ui.horizontal(|ui| {
                ui.label(RichText::new("Audio Visualizer").strong());
                ui.add_space(10.0);

                for style in &[
                    VisualizerStyle::SpectrumBars,
                    VisualizerStyle::Waveform,
                    VisualizerStyle::Circular,
                ] {
                    if ui
                        .selectable_label(state.visualizer_style == *style, style.label())
                        .clicked()
                    {
                        state.visualizer_style = *style;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut state.show_lyrics, "Show Lyrics");
                });
            });

            ui.add_space(4.0);

            // Render live audio visualizer
            render_visualizer(
                ui,
                state.visualizer_style,
                bars,
                peaks,
                raw_samples,
                is_playing,
                140.0,
            );

            ui.add_space(12.0);

            // Synced Scrolling Lyrics Area
            if state.show_lyrics {
                ui.label(RichText::new("Synchronized Lyrics (LRC)").strong());
                ui.add_space(4.0);

                render_lyrics_panel(ui, lyrics, current_pos, seek_request);
            }
        });
    });
}

fn render_lyrics_panel(
    ui: &mut Ui,
    lyrics: Option<&Lyrics>,
    current_pos: f64,
    seek_request: &mut Option<f64>,
) {
    egui::Frame::canvas(ui.style())
        .fill(Color32::from_rgb(16, 18, 24))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(32, 38, 48)))
        .inner_margin(10.0)
        .show(ui, |ui| {
            if let Some(lrc) = lyrics {
                if lrc.lines.is_empty() {
                    ui.label(RichText::new("No lyrics available for this song").italics());
                    return;
                }

                let active_idx = lrc.current_line_index(current_pos);

                egui::ScrollArea::vertical()
                    .max_height(140.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (i, line) in lrc.lines.iter().enumerate() {
                            let is_current = active_idx == Some(i);
                            let text = if is_current {
                                RichText::new(format!("▶ {}", &line.text))
                                    .size(15.0)
                                    .strong()
                                    .color(Color32::from_rgb(255, 175, 45))
                            } else {
                                RichText::new(&line.text)
                                    .size(13.0)
                                    .color(Color32::from_rgb(150, 155, 170))
                            };

                            let resp = ui.selectable_label(is_current, text);
                            if resp.clicked() && lrc.is_synced {
                                *seek_request = Some(line.time_secs);
                            }
                            if is_current {
                                resp.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("No LRC Lyrics Found").color(Color32::GRAY));
                    ui.label(
                        RichText::new("Place a .lrc file with the same name next to the audio file")
                            .small()
                            .color(Color32::DARK_GRAY),
                    );
                    ui.add_space(20.0);
                });
            }
        });
}

fn render_fallback_artwork(ui: &mut Ui, size: Vec2, has_track: bool) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 8.0, Color32::from_rgb(24, 28, 36));
    painter.rect_stroke(rect, 8.0, Stroke::new(1.0_f32, Color32::from_rgb(45, 52, 68)), egui::StrokeKind::Outside);

    let center = rect.center();
    let vinyl_radius = size.x * 0.38;

    // Vinyl grooves
    painter.circle_filled(center, vinyl_radius, Color32::from_rgb(14, 15, 20));
    painter.circle_stroke(
        center,
        vinyl_radius,
        Stroke::new(2.0_f32, Color32::from_rgb(50, 55, 70)),
    );
    painter.circle_stroke(
        center,
        vinyl_radius * 0.75,
        Stroke::new(1.0_f32, Color32::from_rgb(30, 35, 45)),
    );
    painter.circle_stroke(
        center,
        vinyl_radius * 0.5,
        Stroke::new(1.0_f32, Color32::from_rgb(30, 35, 45)),
    );

    // Center vinyl sticker
    let center_color = if has_track {
        Color32::from_rgb(235, 115, 20)
    } else {
        Color32::from_rgb(60, 65, 80)
    };
    painter.circle_filled(center, vinyl_radius * 0.3, center_color);
    painter.circle_filled(center, 5.0, Color32::from_rgb(18, 20, 26));

    // Player Brand Label
    painter.text(
        Pos2::new(center.x, rect.bottom() - 16.0),
        egui::Align2::CENTER_CENTER,
        "XORIN PLAYER",
        egui::FontId::proportional(11.0),
        Color32::from_rgb(130, 140, 160),
    );
}

fn update_album_art(
    ctx: &egui::Context,
    state: &mut NowPlayingUiState,
    metadata: Option<&TrackMetadata>,
) {
    let bytes = metadata.and_then(|m| m.album_art_bytes.as_ref());

    if bytes != state.last_art_bytes.as_ref() {
        state.last_art_bytes = bytes.cloned();

        if let Some(b) = bytes {
            if let Ok(img) = image::load_from_memory(b) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                let color_img = ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    rgba.as_flat_samples().as_slice(),
                );
                state.art_texture = Some(ctx.load_texture("album_art", color_img, Default::default()));
                return;
            }
        }
        state.art_texture = None;
    }
}
