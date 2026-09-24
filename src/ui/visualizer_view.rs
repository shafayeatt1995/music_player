use crate::audio::visualizer::NUM_SPECTRUM_BARS;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualizerStyle {
    SpectrumBars,
    Waveform,
    Circular,
}

impl VisualizerStyle {
    pub fn label(&self) -> &'static str {
        match self {
            VisualizerStyle::SpectrumBars => "Spectrum Bars",
            VisualizerStyle::Waveform => "Oscilloscope Wave",
            VisualizerStyle::Circular => "Circular Radial",
        }
    }
}

pub fn render_visualizer(
    ui: &mut Ui,
    style: VisualizerStyle,
    bars: &[f32; NUM_SPECTRUM_BARS],
    peaks: &[f32; NUM_SPECTRUM_BARS],
    raw_samples: &[f32],
    is_playing: bool,
    height: f32,
) {
    let (rect, _response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), height),
        egui::Sense::hover(),
    );

    let painter = ui.painter_at(rect);

    // Background panel with soft rounded corners and subtle border
    painter.rect_filled(
        rect,
        6.0,
        Color32::from_rgb(18, 20, 26),
    );
    painter.rect_stroke(
        rect,
        6.0,
        Stroke::new(1.0_f32, Color32::from_rgb(35, 40, 52)),
        egui::StrokeKind::Outside,
    );

    match style {
        VisualizerStyle::SpectrumBars => {
            render_spectrum_bars(&painter, rect, bars, peaks, is_playing);
        }
        VisualizerStyle::Waveform => {
            render_waveform(&painter, rect, raw_samples, is_playing);
        }
        VisualizerStyle::Circular => {
            render_circular(&painter, rect, bars, is_playing);
        }
    }
}

fn render_spectrum_bars(
    painter: &egui::Painter,
    rect: Rect,
    bars: &[f32; NUM_SPECTRUM_BARS],
    peaks: &[f32; NUM_SPECTRUM_BARS],
    is_playing: bool,
) {
    let n = NUM_SPECTRUM_BARS;
    let margin = 6.0;
    let avail_width = rect.width() - margin * 2.0;
    let avail_height = rect.height() - margin * 2.0;
    let bar_width = (avail_width / n as f32).max(1.0);
    let gap = (bar_width * 0.2).min(2.5);
    let actual_bar_w = (bar_width - gap).max(1.0);

    for i in 0..n {
        let val = if is_playing { bars[i] } else { 0.02 };
        let peak_val = if is_playing { peaks[i] } else { 0.02 };

        let bar_h = (val * avail_height).clamp(2.0, avail_height);
        let peak_h = (peak_val * avail_height).clamp(2.0, avail_height);

        let x = rect.left() + margin + i as f32 * bar_width;
        let y_bottom = rect.bottom() - margin;
        let y_top = y_bottom - bar_h;
        let y_peak = y_bottom - peak_h;

        // VLC Orange / Amber gradient color calculation
        let t = i as f32 / n as f32;
        let r = (255.0 - t * 30.0) as u8;
        let g = (140.0 + t * 80.0) as u8;
        let b = (30.0 + t * 180.0) as u8;
        let bar_color = Color32::from_rgb(r, g, b);

        let bar_rect = Rect::from_min_max(
            Pos2::new(x, y_top),
            Pos2::new(x + actual_bar_w, y_bottom),
        );
        painter.rect_filled(bar_rect, 2.0, bar_color);

        // Peak cap dot
        if is_playing && peak_val > 0.05 {
            let peak_rect = Rect::from_min_max(
                Pos2::new(x, (y_peak - 2.0).max(rect.top() + margin)),
                Pos2::new(x + actual_bar_w, y_peak.max(rect.top() + margin)),
            );
            painter.rect_filled(peak_rect, 1.0, Color32::from_rgb(255, 230, 140));
        }
    }
}

fn render_waveform(
    painter: &egui::Painter,
    rect: Rect,
    raw_samples: &[f32],
    is_playing: bool,
) {
    let center_y = rect.center().y;
    let amplitude = (rect.height() * 0.42).max(10.0);
    let n = raw_samples.len();

    // Center guideline
    painter.line_segment(
        [
            Pos2::new(rect.left() + 4.0, center_y),
            Pos2::new(rect.right() - 4.0, center_y),
        ],
        Stroke::new(1.0_f32, Color32::from_rgb(30, 36, 48)),
    );

    if n < 2 || !is_playing {
        painter.line_segment(
            [
                Pos2::new(rect.left() + 6.0, center_y),
                Pos2::new(rect.right() - 6.0, center_y),
            ],
            Stroke::new(1.5_f32, Color32::from_rgb(255, 150, 40)),
        );
        return;
    }

    let step = (rect.width() - 12.0) / (n - 1) as f32;
    let points: Vec<Pos2> = raw_samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let x = rect.left() + 6.0 + i as f32 * step;
            let y = center_y - (sample * amplitude).clamp(-amplitude, amplitude);
            Pos2::new(x, y)
        })
        .collect();

    // Neon amber waveform
    let stroke = Stroke::new(2.0_f32, Color32::from_rgb(255, 160, 40));
    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], stroke);
    }
}

fn render_circular(
    painter: &egui::Painter,
    rect: Rect,
    bars: &[f32; NUM_SPECTRUM_BARS],
    is_playing: bool,
) {
    let center = rect.center();
    let base_radius = (rect.height().min(rect.width()) * 0.22).max(15.0);
    let max_spike = (rect.height().min(rect.width()) * 0.22).max(15.0);

    // Inner glowing ring
    painter.circle_stroke(
        center,
        base_radius,
        Stroke::new(2.0_f32, Color32::from_rgb(255, 140, 20)),
    );

    let n = NUM_SPECTRUM_BARS;
    for i in 0..n {
        let angle = (i as f32 / n as f32) * 2.0 * PI - (PI / 2.0);
        let val = if is_playing { bars[i] } else { 0.05 };
        let spike_len = val * max_spike;

        let dir = Vec2::new(angle.cos(), angle.sin());
        let p_start = center + dir * base_radius;
        let p_end = center + dir * (base_radius + spike_len);

        let t = i as f32 / n as f32;
        let r = (255.0 - t * 40.0) as u8;
        let g = (130.0 + t * 90.0) as u8;
        let b = (30.0 + t * 200.0) as u8;

        painter.line_segment(
            [p_start, p_end],
            Stroke::new(2.5_f32, Color32::from_rgb(r, g, b)),
        );
    }
}
