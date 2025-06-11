use egui::{self, Color32, Stroke, vec2};
use egui_typed_input::ValText;
use rustfft::{FftPlanner, num_complex::Complex};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FFTSize {
    Auto,
    Custom(u32),
}

pub struct AliasApp {
    signal_frequency: f32,
    sampling_frequency: f32,
    /// the offset of the signal, between 0 and 2 (must be multiplied with π)
    offset: f32,

    fft_size: FFTSize,

    planner: FftPlanner<f32>,

    memo: AliasAppMemoization,

    show_raw_fft_values: bool,

    frame_count: u64,
}

#[derive(Default)]
pub struct FFTMemoization {
    // input
    sampling_frequency: f32,
    signal_frequency: f32,
    offset: f32,
    fft_signal_size: usize,

    // output
    fft_output: Vec<Complex<f32>>,
}

#[derive(Clone, Default)]
pub struct ReconstructedSignalMemoization {
    // input
    horizontal_pixels: u32,
    sampling_frequency: f32,
    signal_frequency: f32,
    offset: f32,
    fft_len: usize,

    // output
    reconstructed_signal_output: Vec<(f32, f32)>,
}

#[derive(Clone, Default)]
pub struct SignalMemoization {
    // input
    horizontal_pixels: u32,
    signal_frequency: f32,
    offset: f32,

    // output
    signal_output: Vec<(f32, f32)>,
}

#[derive(Clone, Default)]
pub struct SamplePointsMemoization {
    // input
    sampling_frequency: f32,
    signal_frequency: f32,
    offset: f32,

    // output
    sample_points_output: Vec<(f32, f32)>,
}

#[derive(Default)]
pub struct AliasAppMemoization {
    fft: Option<FFTMemoization>,
    reconstructed_signal: Option<ReconstructedSignalMemoization>,
    signal: Option<SignalMemoization>,
    sample_points: Option<SamplePointsMemoization>,
}

impl Default for AliasApp {
    fn default() -> Self {
        Self {
            signal_frequency: 3.0,
            sampling_frequency: 10.0,
            offset: 0.0,
            fft_size: FFTSize::Auto,
            planner: FftPlanner::new(),
            // manual memoization
            memo: AliasAppMemoization::default(),
            show_raw_fft_values: false,
            frame_count: 0,
        }
    }
}

impl AliasApp {
    pub fn ui(&mut self, ctx: &egui::Context) {
        // performance: keep track of frame count and render time
        self.frame_count += 1;
        #[cfg(target_arch = "wasm32")]
        let (performance, render_start_time) = {
            // Get performance object for timing
            let window = web_sys::window().expect("should have window");
            let performance = window
                .performance()
                .expect("should have performance available");
            let start_time = performance.now();
            (performance, start_time)
        };
        #[cfg(not(target_arch = "wasm32"))]
        let render_start_time = std::time::Instant::now();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Set dark mode
            ui.ctx().set_visuals(egui::Visuals::dark());

            self.render_sliders(ui);

            let horizontal_pixels = (ctx.pixels_per_point() * ui.available_width()) as u32;

            // Generate signal points
            let signal = self.calculate_signal(horizontal_pixels);

            // Generate sample points
            let sample_points = self.calculate_sample_points();

            // Constants for all plots
            let _max_y = 1.0;
            let _min_y = -1.0;
            // let y_range = max_y - min_y;

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

                let rect = response.rect;
                ui.painter().rect_filled(rect, 0.0, separator_color);

                ui.add_space(5.0); // Space after separator
            };

            // 1. Original signal with vertical lines at sample points
            self.render_signal_graph(
                ui,
                &signal,
                &sample_points,
                plot_height,
                plot_width,
                draw_axis_labels,
            );
            ui.add_space(5.0);
            draw_separator(ui);

            // 2. Sample points only
            self.render_sample_points_graph(
                ui,
                sample_points,
                plot_height,
                plot_width,
                draw_axis_labels,
            );
            ui.add_space(5.0);
            draw_separator(ui);

            // 3. FFT of sampled points
            let fft_output = self.calculate_fft();
            let fft_size = fft_output.len();
            let freq_resolution = self.sampling_frequency / fft_size as f32;
            // ui.colored_label(
            //     Color32::YELLOW,
            //     format!("FFT(n={fft_size}, resolution={freq_resolution:.4} Hz)"),
            // );
            ui.horizontal(|ui| {
                ui.colored_label(
                    Color32::YELLOW,
                    format!("FFT(n={fft_size}, resolution={freq_resolution:.4} Hz)"),
                );

                if ui.small_button("raw values").clicked() {
                    self.show_raw_fft_values = !self.show_raw_fft_values;
                }
            });
            let response3 = ui.allocate_rect(
                egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::Vec2::new(plot_width, plot_height),
                ),
                egui::Sense::hover(),
            );

            let rect = response3.rect.intersect(ui.clip_rect());
            let painter = ui.painter();

            // Define fixed frequency range (0 to 20 Hz)
            self.render_fft(draw_axis_labels, rect, painter, fft_size, &fft_output);

            if self.show_raw_fft_values {
                self.render_fft_raw_values(ctx, ui, &fft_output);
            }

            ui.add_space(5.0);
            draw_separator(ui);

            // 4. Reconstructed signal

            // Create reconstructed signal
            let recon_signal = self.calculate_reconstructed_signal(horizontal_pixels, &fft_output);

            self.render_reconstructed(
                ui,
                signal,
                plot_height,
                plot_width,
                draw_axis_labels,
                recon_signal,
            );

            // Add extra space before the aliasing warning
            ui.add_space(15.0);

            // Add aliasing warning in its own area below the plot
            if self.signal_frequency >= self.sampling_frequency / 2.0 {
                self.render_aliasing_warning(ui);
            } else {
                // Add some empty space even when there's no warning
                ui.add_space(30.0);
            }

            ui.add_space(15.0);
            draw_separator(ui);

            #[cfg(target_arch = "wasm32")]
            let time = performance.now() - render_start_time;
            #[cfg(not(target_arch = "wasm32"))]
            let time = render_start_time.elapsed().as_secs_f32() * 1000.0;

            ui.label(format!("Frame {}: {:.2} ms", self.frame_count, time));
        });
    }
}

impl AliasApp {
    fn calculate_fft(&mut self) -> Vec<Complex<f32>> {
        let fft_signal_size = match self.fft_size {
            FFTSize::Auto => self.calculate_optimal_fft_size(),
            FFTSize::Custom(size) => {
                let mut s = size as usize;
                if s % 2 != 0 {
                    s += 1;
                }
                s
            }
        };

        match self.memo.fft {
            Some(ref mut memo)
                if memo.sampling_frequency == self.sampling_frequency
                    && memo.signal_frequency == self.signal_frequency
                    && memo.fft_signal_size == fft_signal_size
                    && memo.offset == self.offset =>
            {
                // Use cached FFT output
                memo.fft_output.clone()
            }
            _ => {
                // Calculate FFT and store in memoization
                let fft_output = self._calculate_fft(fft_signal_size);
                self.memo.fft = Some(FFTMemoization {
                    sampling_frequency: self.sampling_frequency,
                    signal_frequency: self.signal_frequency,
                    offset: self.offset,
                    fft_signal_size,
                    fft_output: fft_output.clone(),
                });
                fft_output
            }
        }
    }

    fn calculate_optimal_fft_size(&self) -> usize {
        let mut n = (20.0 * self.sampling_frequency) as usize;
        if n % 2 != 0 {
            n += 1;
        }
        n
    }

    fn _calculate_fft(&mut self, fft_signal_size: usize) -> Vec<Complex<f32>> {
        // Use zero-padding at the beginning and end to reduce edge artifacts
        let n_padding = 0;
        let fft_size = fft_signal_size + 2 * n_padding;

        let mut fft_input = Vec::with_capacity(fft_size);

        // Add zeros at the beginning (pre-padding)
        for _ in 0..n_padding {
            fft_input.push(0.0);
        }

        // Generate the signal with precise frequency and phase
        for i in 0..fft_signal_size {
            let t = i as f32 / self.sampling_frequency;
            // Use more precise sine calculation
            let y = (self.signal_frequency * 2.0 * PI * t + self.offset * PI).sin();
            fft_input.push(y);
        }

        // Add zeros at the end (post-padding)
        for _ in 0..n_padding {
            fft_input.push(0.0);
        }

        // Prepare FFT input
        let fft_input: Vec<Complex<f32>> =
            fft_input.iter().map(|y| Complex::new(*y, 0.0)).collect();

        assert!(fft_input.len() == fft_size);

        // // Apply Blackman-Harris window for even better spectral leakage reduction
        // let window_len = fft_input.len();
        // for i in 0..window_len {
        //     // 4-term Blackman-Harris window has even better sidelobe suppression
        //     let a0 = 0.35875;
        //     let a1 = 0.48829;
        //     let a2 = 0.14128;
        //     let a3 = 0.01168;
        //     let window = a0 - a1 * (2.0 * PI * i as f32 / (window_len - 1) as f32).cos()
        //         + a2 * (4.0 * PI * i as f32 / (window_len - 1) as f32).cos()
        //         - a3 * (6.0 * PI * i as f32 / (window_len - 1) as f32).cos();
        //     fft_input[i] *= window;
        // }

        // Perform FFT
        let planner = &mut self.planner;
        let fft = planner.plan_fft_forward(fft_size);
        let mut fft_output = fft_input.clone();
        fft.process(&mut fft_output);
        fft_output
    }
}

impl AliasApp {
    fn calculate_reconstructed_signal(
        &mut self,
        horizontal_pixels: u32,
        fft_output: &Vec<Complex<f32>>,
    ) -> Vec<(f32, f32)> {
        if let Some(ref memo) = self.memo.reconstructed_signal {
            if memo.horizontal_pixels == horizontal_pixels
                && memo.sampling_frequency == self.sampling_frequency
                && memo.signal_frequency == self.signal_frequency
                && memo.offset == self.offset
                && memo.fft_len == fft_output.len()
            {
                return memo.reconstructed_signal_output.clone();
            }
        }

        let result = self._calculate_reconstructed_signal(horizontal_pixels, &fft_output);

        self.memo.reconstructed_signal = Some(ReconstructedSignalMemoization {
            horizontal_pixels,
            sampling_frequency: self.sampling_frequency,
            signal_frequency: self.signal_frequency,
            offset: self.offset,
            fft_len: fft_output.len(),
            reconstructed_signal_output: result.clone(),
        });

        result
    }

    fn _calculate_reconstructed_signal(
        &mut self,
        horizontal_pixels: u32,
        fft_output: &Vec<Complex<f32>>,
    ) -> Vec<(f32, f32)> {
        let n_recon_points = horizontal_pixels as usize;
        let mut recon_signal = Vec::with_capacity(n_recon_points);
        let fft_size = fft_output.len();
        let freq_resolution = self.sampling_frequency / fft_size as f32;

        assert!(fft_output.len() % 2 == 0);

        for i in 0..n_recon_points {
            let x = i as f32 / n_recon_points as f32 * 2.0 * PI;

            let y = {
                // Reconstruct from FFT data (Inverse Fourier transform simplified)
                let mut y_value = 0.0;

                // Use precise frequency reconstruction formula:
                // y(t) = sum_k [ A_k * cos(2π*f_k*t + φ_k) ]
                for k in 0..fft_size {
                    // the FFT is mirrored
                    let freq = if k <= fft_size / 2 {
                        k as f32 * freq_resolution
                    } else {
                        (k as i32 - fft_size as i32) as f32 * freq_resolution
                    };

                    // Get amplitude and phase from complex FFT output
                    let amplitude: f32 = fft_output[k].norm();
                    let phase: f32 = fft_output[k].arg();

                    // Add this frequency component's contribution at time x
                    if freq > 0.0 {
                        y_value += amplitude * (freq * x + phase).cos() / (fft_size as f32);
                    } else {
                        y_value += amplitude * (-1.0 * freq * x - phase).cos() / (fft_size as f32);
                    }
                }

                y_value
            };

            recon_signal.push((x, y));
        }

        recon_signal
    }
}

impl AliasApp {
    fn calculate_signal(&mut self, horizontal_pixels: u32) -> Vec<(f32, f32)> {
        if let Some(ref memo) = self.memo.signal {
            if memo.horizontal_pixels == horizontal_pixels
                && memo.signal_frequency == self.signal_frequency
                && memo.offset == self.offset
            {
                return memo.signal_output.clone();
            }
        }

        let result = self._calculate_signal(horizontal_pixels);
        self.memo.signal = Some(SignalMemoization {
            horizontal_pixels,
            signal_frequency: self.signal_frequency,
            offset: self.offset,
            signal_output: result.clone(),
        });
        result
    }

    fn _calculate_signal(&mut self, horizontal_pixels: u32) -> Vec<(f32, f32)> {
        let n_signal_points = horizontal_pixels;
        let signal: Vec<(f32, f32)> = (0..n_signal_points)
            .map(|i| {
                let x = i as f32 / n_signal_points as f32 * 2.0 * PI;
                let y = (self.signal_frequency * x + self.offset * PI).sin();
                (x, y)
            })
            .collect();
        signal
    }
}

impl AliasApp {
    fn calculate_sample_points(&mut self) -> Vec<(f32, f32)> {
        if let Some(ref memo) = self.memo.sample_points {
            if memo.sampling_frequency == self.sampling_frequency
                && memo.signal_frequency == self.signal_frequency
                && memo.offset == self.offset
            {
                return memo.sample_points_output.clone();
            }
        }

        let result = self._calculate_sample_points();
        self.memo.sample_points = Some(SamplePointsMemoization {
            sampling_frequency: self.sampling_frequency,
            signal_frequency: self.signal_frequency,
            offset: self.offset,
            sample_points_output: result.clone(),
        });
        result
    }

    fn _calculate_sample_points(&mut self) -> Vec<(f32, f32)> {
        let n_sample_points = self.sampling_frequency as u32 + 1;
        let sample_points: Vec<(f32, f32)> = (0..n_sample_points) // More potential points
            .map(|i| {
                // Fixed calculation: correctly calculate sample points based on sampling frequency
                // Time per sample = 1.0 / sampling_frequency (in seconds)
                // Convert to our x-scale which is in [0, 2π]
                let sample_x = i as f32 * (2.0 * PI / self.sampling_frequency);
                let sample_y = (self.signal_frequency * sample_x + self.offset * PI).sin();
                (sample_x, sample_y)
            })
            .collect();
        sample_points
    }
}

impl AliasApp {
    fn render_sliders(&mut self, ui: &mut egui::Ui) {
        ui.heading("Aliasing Demonstration");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Signal Frequency:");
            ui.spacing_mut().slider_width = ui.available_width() - 100.0;
            ui.add(
                egui::Slider::new(&mut self.signal_frequency, 0.1..=10.0)
                    .text("Hz")
                    .fixed_decimals(2)
                    .step_by(0.01),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Sampling Frequency:");
            ui.spacing_mut().slider_width = ui.available_width() - 100.0;
            ui.add(
                egui::Slider::new(&mut self.sampling_frequency, 0.1..=20.0)
                    .text("Hz")
                    .fixed_decimals(2)
                    .step_by(0.01),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Phase shift:");
            ui.spacing_mut().slider_width = ui.available_width() - 100.0;
            ui.add(
                egui::Slider::new(&mut self.offset, 0.0..=2.0)
                    .text("π rad")
                    .fixed_decimals(2)
                    .step_by(0.01),
            );
        });

        ui.horizontal(|ui| {
            ui.label("FFT Size:");
            let mut is_checked = self.fft_size == FFTSize::Auto;
            if ui
                .add(egui::Checkbox::new(&mut is_checked, "Auto"))
                .changed()
            {
                if is_checked {
                    self.fft_size = FFTSize::Auto;
                } else {
                    self.fft_size = FFTSize::Custom(self.calculate_optimal_fft_size() as u32);
                }
            }
            match self.fft_size {
                FFTSize::Auto => {}
                FFTSize::Custom(size) => {
                    let mut number: ValText<u32, _> = ValText::number_uint();
                    number.set_val(size);
                    if ui.text_edit_singleline(&mut number).changed() {
                        if let Some(Ok(num)) = number.get_val() {
                            self.fft_size = FFTSize::Custom(*num);
                        }
                    }
                }
            }
        });
    }
}

impl AliasApp {
    fn render_signal_graph(
        &mut self,
        ui: &mut egui::Ui,
        signal: &Vec<(f32, f32)>,
        sample_points: &Vec<(f32, f32)>,
        plot_height: f32,
        plot_width: f32,
        draw_axis_labels: impl Fn(&egui::Painter, egui::Rect, &str, &str),
    ) {
        ui.colored_label(
            Color32::YELLOW,
            format!("Signal ({}Hz)", self.signal_frequency),
        );
        let response1 = ui.allocate_rect(
            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(plot_width, plot_height)),
            egui::Sense::hover(),
        );

        let rect = response1.rect.intersect(ui.clip_rect());
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
        for (x, _) in sample_points {
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
        for (x, y) in sample_points {
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

        // draw_axis_labels(painter, rect, "Time", "Amplitude");

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
}

impl AliasApp {
    fn render_sample_points_graph(
        &mut self,
        ui: &mut egui::Ui,
        sample_points: Vec<(f32, f32)>,
        plot_height: f32,
        plot_width: f32,
        draw_axis_labels: impl Fn(&egui::Painter, egui::Rect, &str, &str),
    ) {
        ui.colored_label(Color32::YELLOW, format!("Sample points"));
        let response2 = ui.allocate_rect(
            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(plot_width, plot_height)),
            egui::Sense::hover(),
        );

        let rect = response2.rect.intersect(ui.clip_rect());
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

        // draw_axis_labels(painter, rect, "Time", "Amplitude");
    }
}

impl AliasApp {
    fn render_fft(
        &mut self,
        draw_axis_labels: impl Fn(&egui::Painter, egui::Rect, &str, &str),
        rect: egui::Rect,
        painter: &egui::Painter,
        fft_size: usize,
        fft_output: &Vec<Complex<f32>>,
    ) {
        let max_display_freq = 20.0;

        // Calculate how many points to display for 0-20Hz
        let freq_resolution = self.sampling_frequency / fft_size as f32;

        // Calculate magnitudes
        let magnitudes: Vec<f32> = fft_output //[..display_points]
            .iter()
            .map(|c| c.norm() / fft_size as f32)
            .collect::<Vec<f32>>();

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
                format!("{freq} Hz"),
                egui::FontId::proportional(12.0),
                Color32::YELLOW,
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
                    format!("Alias: {alias_freq:.1} Hz"),
                    egui::FontId::proportional(12.0),
                    Color32::RED,
                );
            }
        }

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
                format!("Nyquist: {nyquist_freq:.1} Hz"),
                egui::FontId::proportional(12.0),
                Color32::YELLOW,
            );
        }

        // Draw FFT bars
        if !magnitudes.is_empty() {
            // For each display bucket, position it according to its frequency
            for i_bucket in 0..magnitudes.len() {
                // Calculate the frequency this bucket represents
                let bucket_freq = i_bucket as f32 * freq_resolution;

                // Position the bucket according to its frequency (scaled to display width)
                let x = rect.left() + (bucket_freq / max_display_freq) * rect.width();

                // Calculate width based on frequency resolution
                let next_freq = (i_bucket + 1) as f32 * freq_resolution;
                let next_x = rect.left() + (next_freq / max_display_freq) * rect.width();
                let bucket_width = f32::max(next_x - x, 1.0);

                let y = magnitudes[i_bucket] * 2.0 * rect.height();

                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::Pos2::new(x, rect.bottom() - y),
                        egui::Pos2::new(x + bucket_width * 0.9, rect.bottom()),
                    ),
                    0.0,
                    Color32::LIGHT_BLUE,
                );
            }
        }

        // draw_axis_labels(painter, rect, "Frequency (Hz)", "Magnitude");
    }
}

impl AliasApp {
    fn render_fft_raw_values(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        fft_output: &Vec<Complex<f32>>,
    ) {
        let fft_size = fft_output.len();
        let freq_resolution = self.sampling_frequency / fft_size as f32;

        egui::Window::new("Raw FFT Values")
            .open(&mut self.show_raw_fft_values)
            .resizable(true)
            .default_width(400.0)
            .max_height(600.0)
            .show(ctx, |ui| {
                ui.label(format!("FFT Size: {}", fft_size));
                ui.label(format!("Frequency Resolution: {:.4} Hz", freq_resolution));
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(500.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Bin");
                            ui.separator();
                            ui.label("Frequency (Hz)");
                            ui.separator();
                            ui.label("Real");
                            ui.separator();
                            ui.label("Imaginary");
                            ui.separator();
                            ui.label("Magnitude / N");
                            ui.separator();
                            ui.label("Phase (rad) / π");
                        });
                        ui.separator();

                        for (i, complex_val) in fft_output.iter().enumerate() {
                            let freq = i as f32 * freq_resolution;
                            let magnitude = complex_val.norm() / fft_size as f32;
                            let phase = complex_val.arg() / PI;

                            ui.horizontal(|ui| {
                                ui.label(format!("{:4}", i));
                                ui.separator();
                                ui.label(format!("{:8.3}", freq));
                                ui.separator();
                                ui.label(format!("{:10.6}", complex_val.re));
                                ui.separator();
                                ui.label(format!("{:10.6}", complex_val.im));
                                ui.separator();
                                ui.label(format!("{:10.6}", magnitude));
                                ui.separator();
                                ui.label(format!("{:8.3}", phase));
                            });
                        }
                    });
            });
    }
}

impl AliasApp {
    fn render_reconstructed(
        &mut self,
        ui: &mut egui::Ui,
        signal: Vec<(f32, f32)>,
        plot_height: f32,
        plot_width: f32,
        draw_axis_labels: impl Fn(&egui::Painter, egui::Rect, &str, &str),
        recon_signal: Vec<(f32, f32)>,
    ) {
        ui.colored_label(Color32::YELLOW, format!("Reconstructed signal"));
        let response4 = ui.allocate_rect(
            egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::new(plot_width, plot_height)),
            egui::Sense::hover(),
        );

        let rect = response4.rect.intersect(ui.clip_rect());
        let painter = ui.painter();

        // Draw horizontal zero line
        painter.line_segment(
            [
                egui::Pos2::new(rect.left(), rect.top() + rect.height() / 2.0),
                egui::Pos2::new(rect.right(), rect.top() + rect.height() / 2.0),
            ],
            Stroke::new(1.0, Color32::YELLOW),
        );

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
                Stroke::new(4.0, Color32::RED),
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

        // Draw sample points
        let sample_points = self.calculate_sample_points();
        for (x, y) in &sample_points {
            painter.circle_filled(
                rect.left_top()
                    + vec2(
                        *x / (2.0 * PI) * rect.width(),
                        rect.height() / 2.0 - y * (rect.height() / 2.0),
                    ),
                4.0,
                Color32::GREEN,
            );
        }

        // draw_axis_labels(painter, rect, "Time", "Amplitude");

        // Add legend
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::Pos2::new(rect.right() - 160.0, rect.top() + 10.0),
                egui::Pos2::new(rect.right() - 10.0, rect.top() + 70.0),
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
            Stroke::new(4.0, Color32::RED),
        );

        painter.circle_filled(
            egui::Pos2::new(rect.right() - 140.0, rect.top() + 60.0),
            4.0,
            Color32::GREEN,
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

        painter.text(
            egui::Pos2::new(rect.right() - 120.0, rect.top() + 60.0),
            egui::Align2::LEFT_CENTER,
            "Sample Points",
            egui::FontId::proportional(12.0),
            Color32::YELLOW,
        );
    }
}

impl AliasApp {
    fn render_aliasing_warning(&mut self, ui: &mut egui::Ui) {
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

                let rect = warning_rect.rect;
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
            });
        });
    }
}
