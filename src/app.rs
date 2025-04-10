use eframe::egui;
use egui::{Color32, Stroke, vec2};
use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

pub struct AliasApp {
    signal_frequency: f32,
    sampling_frequency: f32,
}

impl Default for AliasApp {
    fn default() -> Self {
        Self {
            signal_frequency: 1.0,
            sampling_frequency: 5.0,
        }
    }
}

impl eframe::App for AliasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Set dark mode
            ui.ctx().set_visuals(egui::Visuals::dark());

            ui.heading("Aliasing Demonstration");
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Signal Frequency:");
                ui.add(egui::Slider::new(&mut self.signal_frequency, 0.1..=10.0).text("Hz"));
            });

            ui.horizontal(|ui| {
                ui.label("Sampling Frequency:");
                ui.add(egui::Slider::new(&mut self.sampling_frequency, 0.1..=20.0).text("Hz"));
            });

            // Generate signal points
            let signal_points = 500; // Increased for smoother rendering
            let signal: Vec<(f32, f32)> = (0..signal_points)
                .map(|i| {
                    let x = i as f32 / signal_points as f32 * 2.0 * PI;
                    let y = (self.signal_frequency * x).sin();
                    (x, y)
                })
                .collect();

            // Generate sample points
            let sample_points: Vec<(f32, f32)> = (0..1000) // More potential points
                .filter_map(|i| {
                    // Fixed calculation: correctly calculate sample points based on sampling frequency
                    // Time per sample = 1.0 / sampling_frequency (in seconds)
                    // Convert to our x-scale which is in [0, 2π]
                    let sample_x = i as f32 * (2.0 * PI / self.sampling_frequency);
                    if sample_x <= 2.0 * PI {
                        let sample_y = (self.signal_frequency * sample_x).sin();
                        Some((sample_x, sample_y))
                    } else {
                        None
                    }
                })
                .collect();

            // Constants for all plots
            let max_y = 1.0;
            let min_y = -1.0;
            let y_range = max_y - min_y;

            // Calculate total height needed for all plots
            let plot_height = ui.available_height() / 4.0 - 60.0; // 4 plots with spacing
            let plot_width = ui.available_width();

            // Helper function to draw axis labels
            let draw_axis_labels =
                |painter: &egui::Painter, rect: egui::Rect, x_label: &str, y_label: &str| {
                    // X-axis label
                    painter.text(
                        egui::Pos2::new(rect.right() - 40.0, rect.bottom() + 15.0),
                        egui::Align2::CENTER_CENTER,
                        x_label,
                        egui::FontId::proportional(14.0),
                        Color32::YELLOW,
                    );

                    // Y-axis label
                    painter.text(
                        egui::Pos2::new(rect.left() - 25.0, rect.center().y),
                        egui::Align2::CENTER_CENTER,
                        y_label,
                        egui::FontId::proportional(14.0),
                        Color32::YELLOW,
                    );
                };

            // Helper function to draw a separator line
            let draw_separator = |ui: &mut egui::Ui| {
                let separator_height = 2.0;
                let separator_color = Color32::from_rgb(100, 100, 100); // Dark gray

                let response = ui.allocate_rect(
                    egui::Rect::from_min_size(
                        ui.cursor().min,
                        egui::Vec2::new(ui.available_width(), separator_height),
                    ),
                    egui::Sense::hover(),
                );

                if let rect = response.rect {
                    ui.painter().rect_filled(rect, 0.0, separator_color);
                }

                ui.add_space(5.0); // Space after separator
            };

            // 1. Original signal with vertical lines at sample points
            ui.colored_label(
                Color32::YELLOW,
                format!(
                    "Original signal ({}Hz) with sample points ({}Hz)",
                    self.signal_frequency, self.sampling_frequency
                ),
            );
            let response1 = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::Vec2::new(plot_width, plot_height),
                ),
                egui::Sense::hover(),
            );

            if let rect = response1.rect.intersect(ui.clip_rect()) {
                let painter = ui.painter();

                // Draw signal
                for i in 0..signal.len() - 1 {
                    let (x1, y1) = signal[i];
                    let (x2, y2) = signal[i + 1];
                    painter.line_segment(
                        [
                            rect.left_top()
                                + vec2(
                                    x1 / (2.0 * PI) * rect.width(),
                                    rect.height() / 2.0 - y1 * (rect.height() / 2.0),
                                ),
                            rect.left_top()
                                + vec2(
                                    x2 / (2.0 * PI) * rect.width(),
                                    rect.height() / 2.0 - y2 * (rect.height() / 2.0),
                                ),
                        ],
                        Stroke::new(2.0, Color32::GREEN),
                    );
                }

                // Draw vertical lines at sample points
                for (x, _) in &sample_points {
                    let x_pos = rect.left_top().x + *x / (2.0 * PI) * rect.width();
                    painter.line_segment(
                        [
                            egui::Pos2::new(x_pos, rect.top()),
                            egui::Pos2::new(x_pos, rect.bottom()),
                        ],
                        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 0, 0, 100)),
                    );
                }

                // Draw sample points
                for (x, y) in &sample_points {
                    painter.circle_filled(
                        rect.left_top()
                            + vec2(
                                *x / (2.0 * PI) * rect.width(),
                                rect.height() / 2.0 - y * (rect.height() / 2.0),
                            ),
                        4.0,
                        Color32::RED,
                    );
                }

                draw_axis_labels(painter, rect, "Time", "Amplitude");

                // Add legend
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::Pos2::new(rect.right() - 120.0, rect.top() + 10.0),
                        egui::Pos2::new(rect.right() - 10.0, rect.top() + 50.0),
                    ),
                    3.0,
                    Color32::from_rgba_premultiplied(40, 40, 40, 200),
                );

                painter.line_segment(
                    [
                        egui::Pos2::new(rect.right() - 110.0, rect.top() + 20.0),
                        egui::Pos2::new(rect.right() - 90.0, rect.top() + 20.0),
                    ],
                    Stroke::new(2.0, Color32::GREEN),
                );

                painter.circle_filled(
                    egui::Pos2::new(rect.right() - 100.0, rect.top() + 40.0),
                    4.0,
                    Color32::RED,
                );

                painter.text(
                    egui::Pos2::new(rect.right() - 80.0, rect.top() + 20.0),
                    egui::Align2::LEFT_CENTER,
                    "Signal",
                    egui::FontId::proportional(12.0),
                    Color32::YELLOW,
                );

                painter.text(
                    egui::Pos2::new(rect.right() - 80.0, rect.top() + 40.0),
                    egui::Align2::LEFT_CENTER,
                    "Samples",
                    egui::FontId::proportional(12.0),
                    Color32::YELLOW,
                );
            }
            ui.add_space(5.0);
            draw_separator(ui);

            // 2. Sample points only
            ui.colored_label(
                Color32::YELLOW,
                format!(
                    "Sample points only (Sampling rate: {}Hz)",
                    self.sampling_frequency
                ),
            );
            let response2 = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::Vec2::new(plot_width, plot_height),
                ),
                egui::Sense::hover(),
            );

            if let rect = response2.rect.intersect(ui.clip_rect()) {
                let painter = ui.painter();

                // Draw horizontal zero line
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.left(), rect.top() + rect.height() / 2.0),
                        egui::Pos2::new(rect.right(), rect.top() + rect.height() / 2.0),
                    ],
                    Stroke::new(1.0, Color32::YELLOW),
                );

                // Draw sample points
                for (x, y) in &sample_points {
                    painter.circle_filled(
                        rect.left_top()
                            + vec2(
                                *x / (2.0 * PI) * rect.width(),
                                rect.height() / 2.0 - y * (rect.height() / 2.0),
                            ),
                        4.0,
                        Color32::RED,
                    );
                }

                draw_axis_labels(painter, rect, "Time", "Amplitude");

                // Add nyquist frequency label
                let nyquist_freq = self.sampling_frequency / 2.0;
                painter.text(
                    egui::Pos2::new(rect.left() + 10.0, rect.top() + 15.0),
                    egui::Align2::LEFT_CENTER,
                    format!("Nyquist frequency: {:.1} Hz", nyquist_freq),
                    egui::FontId::proportional(12.0),
                    Color32::YELLOW,
                );
            }
            ui.add_space(5.0);
            draw_separator(ui);

            // 3. FFT of sampled points
            ui.colored_label(
                Color32::YELLOW,
                "FFT of sampled points (Frequency spectrum)",
            );
            let response3 = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::Vec2::new(plot_width, plot_height),
                ),
                egui::Sense::hover(),
            );

            if let rect = response3.rect.intersect(ui.clip_rect()) {
                let painter = ui.painter();

                // Prepare data for FFT with even higher precision
                let fft_size = 16384; // Increased for better resolution

                // Create even longer signal for precision
                let extended_signal_len = 20000; // Double the previous length
                let mut extended_signal = Vec::with_capacity(extended_signal_len);

                // Generate more cycles of the signal for better frequency resolution
                let cycles_to_simulate = 100.0; // Increased to 100 cycles

                // Use zero-padding at the beginning and end to reduce edge artifacts
                let padding = 1000; // Add 1000 zeros at beginning and end

                // Add zeros at the beginning (pre-padding)
                for _ in 0..padding {
                    extended_signal.push(0.0);
                }

                // Generate the signal with precise frequency and phase
                for i in 0..extended_signal_len {
                    let t = i as f32 / self.sampling_frequency;
                    if t * self.signal_frequency < cycles_to_simulate {
                        // Use more precise sine calculation
                        let y = (self.signal_frequency * 2.0 * PI * t).sin();
                        extended_signal.push(y);
                    } else {
                        break;
                    }
                }

                // Add zeros at the end (post-padding)
                for _ in 0..padding {
                    extended_signal.push(0.0);
                }

                // Prepare FFT input
                let mut fft_input: Vec<Complex<f32>> = extended_signal
                    .iter()
                    .map(|y| Complex::new(*y, 0.0))
                    .collect();

                // Pad with zeros to the full FFT size
                fft_input.resize(fft_size, Complex::new(0.0, 0.0));

                // Apply Blackman-Harris window for even better spectral leakage reduction
                let window_len = fft_input.len();
                for i in 0..window_len {
                    // 4-term Blackman-Harris window has even better sidelobe suppression
                    let a0 = 0.35875;
                    let a1 = 0.48829;
                    let a2 = 0.14128;
                    let a3 = 0.01168;
                    let window = a0 - a1 * (2.0 * PI * i as f32 / (window_len - 1) as f32).cos()
                        + a2 * (4.0 * PI * i as f32 / (window_len - 1) as f32).cos()
                        - a3 * (6.0 * PI * i as f32 / (window_len - 1) as f32).cos();
                    fft_input[i] *= window;
                }

                // Perform FFT
                let mut planner = FftPlanner::new();
                let fft = planner.plan_fft_forward(fft_size);
                let mut fft_output = fft_input.clone();
                fft.process(&mut fft_output);

                // Define fixed frequency range (0 to 20 Hz)
                let max_display_freq = 20.0;

                // Calculate how many points to display for 0-20Hz
                let freq_resolution = self.sampling_frequency / fft_size as f32;
                let display_points = (max_display_freq / freq_resolution).ceil() as usize;
                let display_points = display_points.min(fft_size / 2); // Don't exceed Nyquist

                // Define number of buckets for display (more buckets = more resolution)
                let num_buckets = 200; // Increased from before

                // Calculate magnitudes
                let magnitudes: Vec<f32> = fft_output[..display_points]
                    .iter()
                    .map(|c| c.norm() / fft_size as f32)
                    .collect();

                // Find maximum for scaling
                let max_magnitude = magnitudes.iter().fold(0.0f32, |a, &b| a.max(b));

                // Draw horizontal zero line
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.left(), rect.bottom()),
                        egui::Pos2::new(rect.right(), rect.bottom()),
                    ],
                    Stroke::new(1.0, Color32::YELLOW),
                );

                // Draw vertical axis line
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.left(), rect.top()),
                        egui::Pos2::new(rect.left(), rect.bottom()),
                    ],
                    Stroke::new(1.0, Color32::YELLOW),
                );

                // Draw frequency ticks every 5 Hz
                for freq in (0..=20).step_by(5) {
                    let x_pos = rect.left() + (freq as f32 / max_display_freq) * rect.width();

                    // Draw tick
                    painter.line_segment(
                        [
                            egui::Pos2::new(x_pos, rect.bottom()),
                            egui::Pos2::new(x_pos, rect.bottom() + 5.0),
                        ],
                        Stroke::new(1.0, Color32::YELLOW),
                    );

                    // Draw tick label
                    painter.text(
                        egui::Pos2::new(x_pos, rect.bottom() + 15.0),
                        egui::Align2::CENTER_CENTER,
                        format!("{} Hz", freq),
                        egui::FontId::proportional(12.0),
                        Color32::YELLOW,
                    );
                }

                // Draw FFT bars
                if !magnitudes.is_empty() && max_magnitude > 0.0 {
                    // Calculate frequency for each bucket in our display range
                    let bucket_width = rect.width() / num_buckets as f32;

                    // For each display bucket, find the max magnitude in the corresponding frequency range
                    for bucket in 0..num_buckets {
                        let bucket_start_freq =
                            bucket as f32 * max_display_freq / num_buckets as f32;
                        let bucket_end_freq =
                            (bucket + 1) as f32 * max_display_freq / num_buckets as f32;

                        let bucket_start_idx =
                            (bucket_start_freq / freq_resolution).floor() as usize;
                        let bucket_end_idx = (bucket_end_freq / freq_resolution).ceil() as usize;

                        let bucket_end_idx = bucket_end_idx.min(display_points);

                        // Find max magnitude in this frequency bucket
                        let mut max_bucket_magnitude: f32 = 0.0;
                        if bucket_start_idx < bucket_end_idx && bucket_start_idx < magnitudes.len()
                        {
                            for i in bucket_start_idx..bucket_end_idx.min(magnitudes.len()) {
                                max_bucket_magnitude = max_bucket_magnitude.max(magnitudes[i]);
                            }
                        }

                        let normalized_height =
                            max_bucket_magnitude / max_magnitude * rect.height();
                        let x = rect.left() + bucket as f32 * bucket_width;

                        painter.rect_filled(
                            egui::Rect::from_min_max(
                                egui::Pos2::new(x, rect.bottom() - normalized_height),
                                egui::Pos2::new(x + bucket_width * 0.9, rect.bottom()),
                            ),
                            0.0,
                            Color32::LIGHT_BLUE,
                        );
                    }

                    // Mark signal frequency position
                    let signal_freq_pos = (self.signal_frequency / max_display_freq) * rect.width();
                    if signal_freq_pos <= rect.width() {
                        painter.line_segment(
                            [
                                egui::Pos2::new(rect.left() + signal_freq_pos, rect.top()),
                                egui::Pos2::new(rect.left() + signal_freq_pos, rect.bottom()),
                            ],
                            Stroke::new(1.0, Color32::RED),
                        );

                        painter.text(
                            egui::Pos2::new(rect.left() + signal_freq_pos, rect.top() + 15.0),
                            egui::Align2::CENTER_CENTER,
                            format!("{:.1} Hz", self.signal_frequency),
                            egui::FontId::proportional(12.0),
                            Color32::YELLOW,
                        );
                    }

                    // Add aliased frequency label if applicable
                    if self.signal_frequency > self.sampling_frequency / 2.0 {
                        let alias_freq = self.signal_frequency % self.sampling_frequency;
                        let alias_freq = if alias_freq > self.sampling_frequency / 2.0 {
                            self.sampling_frequency - alias_freq
                        } else {
                            alias_freq
                        };

                        let alias_pos = (alias_freq / max_display_freq) * rect.width();
                        if alias_pos <= rect.width() {
                            painter.line_segment(
                                [
                                    egui::Pos2::new(rect.left() + alias_pos, rect.top()),
                                    egui::Pos2::new(rect.left() + alias_pos, rect.bottom()),
                                ],
                                Stroke::new(1.0, Color32::from_rgb(128, 0, 128)), // Purple
                            );

                            painter.text(
                                egui::Pos2::new(rect.left() + alias_pos + 50.0, rect.top() + 30.0),
                                egui::Align2::CENTER_CENTER,
                                format!("Alias: {:.1} Hz", alias_freq),
                                egui::FontId::proportional(12.0),
                                Color32::ORANGE,
                            );
                        }
                    }
                }

                draw_axis_labels(painter, rect, "Frequency (Hz)", "Magnitude");

                // Mark Nyquist frequency if it's in our display range
                let nyquist_freq = self.sampling_frequency / 2.0;
                if nyquist_freq <= max_display_freq {
                    let nyquist_pos = (nyquist_freq / max_display_freq) * rect.width();
                    painter.line_segment(
                        [
                            egui::Pos2::new(rect.left() + nyquist_pos, rect.top()),
                            egui::Pos2::new(rect.left() + nyquist_pos, rect.bottom()),
                        ],
                        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 0, 100)),
                    );

                    painter.text(
                        egui::Pos2::new(rect.left() + nyquist_pos, rect.bottom() - 5.0),
                        egui::Align2::CENTER_BOTTOM,
                        format!("Nyquist: {:.1} Hz", nyquist_freq),
                        egui::FontId::proportional(12.0),
                        Color32::YELLOW,
                    );
                }
            }
            ui.add_space(5.0);
            draw_separator(ui);

            // 4. Reconstructed signal
            ui.colored_label(
                Color32::YELLOW,
                format!(
                    "Reconstructed signal ({}Hz sampling)",
                    self.sampling_frequency
                ),
            );
            let response4 = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::Vec2::new(plot_width, plot_height),
                ),
                egui::Sense::hover(),
            );

            if let rect = response4.rect.intersect(ui.clip_rect()) {
                let painter = ui.painter();

                // Draw horizontal zero line
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.left(), rect.top() + rect.height() / 2.0),
                        egui::Pos2::new(rect.right(), rect.top() + rect.height() / 2.0),
                    ],
                    Stroke::new(1.0, Color32::YELLOW),
                );

                // Create reconstructed signal using sinc interpolation
                let recon_points = 400; // Double for smoother reconstruction
                let mut recon_signal = Vec::with_capacity(recon_points);

                // Calculate the time period between samples
                let sample_period = 1.0 / self.sampling_frequency;

                // Enhanced sinc interpolation with more accurate reconstruction
                // We'll use all sample points, but apply a window function to reduce edge effects
                for i in 0..recon_points {
                    let x = i as f32 / recon_points as f32 * 2.0 * PI;

                    // Sinc interpolation with proper scaling
                    let mut y = 0.0;

                    // Count total samples for normalization
                    let total_samples = sample_points.len();

                    for (j, (sample_x, sample_y)) in sample_points.iter().enumerate() {
                        // Calculate time difference and scale by sampling frequency
                        let dt = x - sample_x;
                        let normalized_dt = dt * self.sampling_frequency / (2.0 * PI);

                        // Apply window function based on sample distance from edges
                        // This reduces edge effects in the reconstruction
                        let edge_distance = (j as f32 / total_samples as f32 - 0.5).abs() * 2.0;
                        let window = 0.5 * (1.0 - (PI * edge_distance).cos()); // Hann window

                        // Optimized sinc calculation with protection against division by zero
                        let sinc = if normalized_dt.abs() < 1e-6 {
                            1.0
                        } else {
                            (PI * normalized_dt).sin() / (PI * normalized_dt)
                        };

                        // Apply both window and sinc
                        y += sample_y * sinc;
                    }

                    recon_signal.push((x, y));
                }

                // Draw reconstructed signal
                for i in 0..recon_signal.len() - 1 {
                    let (x1, y1) = recon_signal[i];
                    let (x2, y2) = recon_signal[i + 1];

                    painter.line_segment(
                        [
                            rect.left_top()
                                + vec2(
                                    x1 / (2.0 * PI) * rect.width(),
                                    rect.height() / 2.0 - y1 * (rect.height() / 2.0),
                                ),
                            rect.left_top()
                                + vec2(
                                    x2 / (2.0 * PI) * rect.width(),
                                    rect.height() / 2.0 - y2 * (rect.height() / 2.0),
                                ),
                        ],
                        Stroke::new(2.0, Color32::RED),
                    );
                }

                // Draw original signal for comparison (thinner line)
                for i in 0..signal.len() - 1 {
                    let (x1, y1) = signal[i];
                    let (x2, y2) = signal[i + 1];
                    painter.line_segment(
                        [
                            rect.left_top()
                                + vec2(
                                    x1 / (2.0 * PI) * rect.width(),
                                    rect.height() / 2.0 - y1 * (rect.height() / 2.0),
                                ),
                            rect.left_top()
                                + vec2(
                                    x2 / (2.0 * PI) * rect.width(),
                                    rect.height() / 2.0 - y2 * (rect.height() / 2.0),
                                ),
                        ],
                        Stroke::new(1.0, Color32::GREEN),
                    );
                }

                draw_axis_labels(painter, rect, "Time", "Amplitude");

                // Add legend
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::Pos2::new(rect.right() - 160.0, rect.top() + 10.0),
                        egui::Pos2::new(rect.right() - 10.0, rect.top() + 50.0),
                    ),
                    3.0,
                    Color32::from_rgba_premultiplied(40, 40, 40, 200),
                );

                painter.line_segment(
                    [
                        egui::Pos2::new(rect.right() - 150.0, rect.top() + 20.0),
                        egui::Pos2::new(rect.right() - 130.0, rect.top() + 20.0),
                    ],
                    Stroke::new(1.0, Color32::GREEN),
                );

                painter.line_segment(
                    [
                        egui::Pos2::new(rect.right() - 150.0, rect.top() + 40.0),
                        egui::Pos2::new(rect.right() - 130.0, rect.top() + 40.0),
                    ],
                    Stroke::new(2.0, Color32::RED),
                );

                painter.text(
                    egui::Pos2::new(rect.right() - 120.0, rect.top() + 20.0),
                    egui::Align2::LEFT_CENTER,
                    "Original Signal",
                    egui::FontId::proportional(12.0),
                    Color32::YELLOW,
                );

                painter.text(
                    egui::Pos2::new(rect.right() - 120.0, rect.top() + 40.0),
                    egui::Align2::LEFT_CENTER,
                    "Reconstructed",
                    egui::FontId::proportional(12.0),
                    Color32::YELLOW,
                );
            }

            // Add extra space before the aliasing warning
            ui.add_space(15.0);

            // Add aliasing warning in its own area below the plot
            if self.signal_frequency > self.sampling_frequency / 2.0 {
                let alias_freq = self.signal_frequency % self.sampling_frequency;
                let alias_freq = if alias_freq > self.sampling_frequency / 2.0 {
                    self.sampling_frequency - alias_freq
                } else {
                    alias_freq
                };

                ui.horizontal(|ui| {
                    // Add a bit of padding on the left
                    ui.add_space(10.0);

                    ui.vertical(|ui| {
                        let warning_rect = ui.allocate_rect(
                            egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::Vec2::new(ui.available_width() - 20.0, 60.0),
                            ),
                            egui::Sense::hover(),
                        );

                        if let rect = warning_rect.rect {
                            ui.painter().rect_filled(
                                rect,
                                5.0,
                                Color32::from_rgba_premultiplied(100, 0, 0, 200),
                            );

                            ui.painter().text(
                                egui::Pos2::new(rect.left() + 20.0, rect.top() + 20.0),
                                egui::Align2::LEFT_CENTER,
                                "Aliasing detected! Signal frequency exceeds Nyquist limit.",
                                egui::FontId::proportional(16.0),
                                Color32::RED,
                            );

                            ui.painter().text(
                                egui::Pos2::new(rect.left() + 20.0, rect.top() + 40.0),
                                egui::Align2::LEFT_CENTER,
                                format!(
                                    "Signal: {:.1} Hz appears as: {:.1} Hz (Nyquist: {:.1} Hz)",
                                    self.signal_frequency,
                                    alias_freq,
                                    self.sampling_frequency / 2.0
                                ),
                                egui::FontId::proportional(14.0),
                                Color32::LIGHT_RED,
                            );
                        }
                    });
                });
            } else {
                // Add some empty space even when there's no warning
                ui.add_space(30.0);
            }

            ui.add_space(15.0);
            draw_separator(ui);
        });
    }
}
