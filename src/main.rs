// src/main.rs

use eframe::egui::{self, Color32, FontId, Pos2, Rect, TextureHandle, Vec2};
use eframe::epaint::ColorImage;
use image;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source}; // Audio imports
use std::io::Cursor;
use std::time::{Duration, Instant};

// Pre-defined coordinates for drawing the numbers on the limbs.
const FINGER_POSITIONS: [(f32, f32); 10] = [
    (0.08, 0.40), (0.16, 0.25), (0.25, 0.18), (0.34, 0.28), (0.43, 0.60), // Left Hand
    (0.57, 0.60), (0.66, 0.28), (0.75, 0.18), (0.84, 0.25), (0.92, 0.40), // Right Hand
];

// Final toe positions with left-to-right counting order.
const TOE_POSITIONS: [(f32, f32); 10] = [
    (0.21, 0.18), (0.28, 0.15), (0.31, 0.13), (0.38, 0.11), (0.46, 0.08), // Left Foot
    (0.51, 0.08), (0.65, 0.11), (0.68, 0.13), (0.72, 0.15), (0.79, 0.18), // Right Foot
];

// --- Enums and Structs ---
enum AppView { Calculator, Animation, Result }
struct AnimationState { num1: u8, num2: u8, total: u8, current_count: u8, last_update: Instant }
struct Textures { hands: TextureHandle, feet: TextureHandle }

struct LiteralCalculatorApp {
    input_string: String,
    current_view: AppView,
    animation_state: Option<AnimationState>,
    result_message: String,
    textures: Textures,
    // --- Audio fields ---
    _stream: OutputStream, // Must be kept alive to play audio
    stream_handle: OutputStreamHandle,
    audio_files: Vec<&'static [u8]>,
}

impl LiteralCalculatorApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().expect("Failed to find audio output device");

        // --- Load Audio Files into Memory, CASTING EACH TO A SLICE ---
        let audio_files: Vec<&'static [u8]> = vec![
            include_bytes!("../assets/audio/1.mp3") as &'static [u8],
            include_bytes!("../assets/audio/2.mp3") as &'static [u8],
            include_bytes!("../assets/audio/3.mp3") as &'static [u8],
            include_bytes!("../assets/audio/4.mp3") as &'static [u8],
            include_bytes!("../assets/audio/5.mp3") as &'static [u8],
            include_bytes!("../assets/audio/6.mp3") as &'static [u8],
            include_bytes!("../assets/audio/7.mp3") as &'static [u8],
            include_bytes!("../assets/audio/8.mp3") as &'static [u8],
            include_bytes!("../assets/audio/9.mp3") as &'static [u8],
            include_bytes!("../assets/audio/10.mp3") as &'static [u8],
            include_bytes!("../assets/audio/11.mp3") as &'static [u8],
            include_bytes!("../assets/audio/12.mp3") as &'static [u8],
            include_bytes!("../assets/audio/13.mp3") as &'static [u8],
            include_bytes!("../assets/audio/14.mp3") as &'static [u8],
            include_bytes!("../assets/audio/15.mp3") as &'static [u8],
            include_bytes!("../assets/audio/16.mp3") as &'static [u8],
            include_bytes!("../assets/audio/17.mp3") as &'static [u8],
            include_bytes!("../assets/audio/18.mp3") as &'static [u8],
            include_bytes!("../assets/audio/19.mp3") as &'static [u8],
            include_bytes!("../assets/audio/20.mp3") as &'static [u8],
        ];

        let textures = {
            let hands_bytes = include_bytes!("../assets/images/hands.png");
            let feet_bytes = include_bytes!("../assets/images/feet.png");
            let hands_image = image::load_from_memory(hands_bytes).expect("Failed to load hands.png");
            let feet_image = image::load_from_memory(feet_bytes).expect("Failed to load feet.png");
            let hands_size = [hands_image.width() as _, hands_image.height() as _];
            let feet_size = [feet_image.width() as _, feet_image.height() as _];
            let hands_rgba = hands_image.to_rgba8();
            let feet_rgba = feet_image.to_rgba8();
            let hands_img = ColorImage::from_rgba_unmultiplied(hands_size, &hands_rgba);
            let feet_img = ColorImage::from_rgba_unmultiplied(feet_size, &feet_rgba);
            Textures {
                hands: cc.egui_ctx.load_texture("hands", hands_img, Default::default()),
                feet: cc.egui_ctx.load_texture("feet", feet_img, Default::default()),
            }
        };

        Self {
            input_string: String::new(),
            current_view: AppView::Calculator,
            animation_state: None,
            result_message: String::new(),
            textures,
            _stream,
            stream_handle,
            audio_files,
        }
    }
}

impl eframe::App for LiteralCalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_view {
                AppView::Calculator => self.show_calculator_ui(ui),
                AppView::Animation => self.show_animation_ui(ui),
                AppView::Result => self.show_result_ui(ui),
            }
        });
    }
}

impl LiteralCalculatorApp {
    fn show_calculator_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Sum-thing's Afoot Calculator");
            ui.add_space(20.0);
            ui.label("Enter addition (e.g., 3+3):");
            ui.text_edit_singleline(&mut self.input_string);
            ui.add_space(10.0);
            if ui.button("Count!").clicked() {
                let parts: Vec<&str> = self.input_string.split('+').map(|s| s.trim()).collect();
                if parts.len() == 2 {
                    if let (Ok(num1), Ok(num2)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                        let total = num1 as u16 + num2 as u16;
                        if total > 20 {
                            self.result_message = format!("Error: Sum ({}) is greater than 20. I've run out of limbs!", total);
                            self.current_view = AppView::Result;
                        } else {
                            self.animation_state = Some(AnimationState { num1, num2, total: total as u8, current_count: 0, last_update: Instant::now() });
                            self.current_view = AppView::Animation;
                        }
                    } else { self.result_message = "Error: Invalid numbers.".to_string(); self.current_view = AppView::Result; }
                } else { self.result_message = "Error: Use the format 'number+number'.".to_string(); self.current_view = AppView::Result; }
            }
        });
    }

    fn show_animation_ui(&mut self, ui: &mut egui::Ui) {
        if let Some(state) = &mut self.animation_state {
            ui.heading("Counting...");
            ui.add_space(10.0);
            let available_width = ui.available_width();
            let hands_aspect_ratio = self.textures.hands.size_vec2().y / self.textures.hands.size_vec2().x;
            let hands_size = Vec2::new(available_width, available_width * hands_aspect_ratio);
            let hands_rect = ui.allocate_exact_size(hands_size, egui::Sense::hover()).0;
            ui.painter().image(self.textures.hands.id(), hands_rect, Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)), Color32::WHITE);
            let show_feet = state.total > 10;
            let mut feet_rect = Rect::ZERO;
            if show_feet {
                ui.add_space(10.0);
                let feet_aspect_ratio = self.textures.feet.size_vec2().y / self.textures.feet.size_vec2().x;
                let feet_size = Vec2::new(available_width, available_width * feet_aspect_ratio);
                feet_rect = ui.allocate_exact_size(feet_size, egui::Sense::hover()).0;
                ui.painter().image(self.textures.feet.id(), feet_rect, Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)), Color32::WHITE);
            }
            for i in 1..=state.current_count {
                let num_str = i.to_string();
                let color = if i <= state.num1 { Color32::from_rgb(255, 50, 50) } else { Color32::from_rgb(50, 255, 50) };
                let (img_rect, pos) = if i <= 10 {
                    (hands_rect, FINGER_POSITIONS[(i - 1) as usize])
                } else {
                    (feet_rect, TOE_POSITIONS[(i - 11) as usize])
                };
                let draw_pos = Pos2::new(img_rect.min.x + img_rect.width() * pos.0, img_rect.min.y + img_rect.height() * pos.1);
                ui.painter().text(draw_pos, egui::Align2::CENTER_CENTER, num_str, FontId::proportional(30.0), color);
            }
            if state.last_update.elapsed() > Duration::from_millis(600) {
                if state.current_count < state.total {
                    state.current_count += 1;
                    let audio_data = self.audio_files[(state.current_count - 1) as usize];
                    let source = Decoder::new(Cursor::new(audio_data)).unwrap();
                    let _ = self.stream_handle.play_raw(source.convert_samples());
                    state.last_update = Instant::now(); // Reset timer here after playing sound
                } else {
                    self.result_message = format!("{} + {} = {}", state.num1, state.num2, state.total);
                    self.current_view = AppView::Result;
                }
            }
            ui.ctx().request_repaint();
        }
    }

    fn show_result_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Result");
            ui.add_space(20.0);
            ui.label(&self.result_message);
            ui.add_space(10.0);
            if ui.button("New Calculation").clicked() {
                self.input_string.clear();
                self.animation_state = None;
                self.current_view = AppView::Calculator;
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("The Literal Calculator", options, Box::new(|cc| Box::new(LiteralCalculatorApp::new(cc)))).expect("Failed to run eframe");
}