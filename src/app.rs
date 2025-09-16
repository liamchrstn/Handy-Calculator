// src/app.rs

use eframe::{egui, epi};
use std::time::{Duration, Instant};

// Enum to represent the current screen being displayed.
enum AppView {
    Calculator,
    Animation,
    Result,
}

// State for the counting animation.
struct AnimationState {
    num1: u8,
    num2: u8,
    total: u8,
    current_count: u8,
    last_update: Instant,
    // Add audio and image handles here
}

pub struct LiteralCalculatorApp {
    input_string: String,
    current_view: AppView,
    animation_state: Option<AnimationState>,
    result_message: String,
}

impl LiteralCalculatorApp {
    pub fn new() -> Self {
        Self {
            input_string: String::new(),
            current_view: AppView::Calculator,
            animation_state: None,
            result_message: String::new(),
        }
    }
}

// This is where we implement the eframe::App trait, which is the heart of our egui app.
impl eframe::App for LiteralCalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Central panel for our UI.
        egui::CentralPanel::default().show(ctx, |ui| {
            // We use a match statement to render different UIs for different views.
            match self.current_view {
                AppView::Calculator => self.show_calculator_ui(ui),
                AppView::Animation => self.show_animation_ui(ui),
                AppView::Result => self.show_result_ui(ui),
            }
        });
    }
}

// UI-specific methods for our app struct.
impl LiteralCalculatorApp {
    /// Renders the main calculator interface.
    fn show_calculator_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("The Literal Calculator");
        ui.add_space(20.0);

        // Input field for the equation
        ui.text_edit_singleline(&mut self.input_string);
        ui.add_space(10.0);

        // "Count!" button
        if ui.button("Count!").clicked() {
            // Parse the input string like "6+5"
            let parts: Vec<&str> = self.input_string.split('+').collect();
            if parts.len() == 2 {
                if let (Ok(num1), Ok(num2)) = (parts[0].trim().parse::<u8>(), parts[1].trim().parse::<u8>()) {
                    let total = num1 + num2;
                    if total > 20 {
                        self.result_message = "Error: I've run out of limbs!".to_string();
                        self.current_view = AppView::Result;
                    } else {
                        // If parsing is successful, create a new animation state
                        self.animation_state = Some(AnimationState {
                            num1,
                            num2,
                            total,
                            current_count: 0,
                            last_update: Instant::now(),
                        });
                        // Switch to the animation view
                        self.current_view = AppView::Animation;
                    }
                }
            }
        }
    }

    /// Renders the counting animation.
    fn show_animation_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Counting...");
        ui.add_space(20.0);

        if let Some(state) = &mut self.animation_state {
            ui.label(format!("Counting to: {}", state.total));
            ui.label(format!("Current number: {}", state.current_count));

            // Logic to advance the animation frame every 0.75 seconds
            if state.last_update.elapsed() > Duration::from_millis(750) {
                if state.current_count < state.total {
                    state.current_count += 1;
                    state.last_update = Instant::now();
                    // *** TODO: Play audio for `state.current_count` here using rodio ***
                } else {
                    // Animation is finished, switch to the result view
                    self.result_message = format!("{} + {} = {}", state.num1, state.num2, state.total);
                    self.current_view = AppView::Result;
                }
            }
            // Request a repaint to keep the animation running
            ui.ctx().request_repaint();
        }
    }

    /// Renders the final result pop-up.
    fn show_result_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Result");
        ui.add_space(20.0);

        ui.label(&self.result_message);
        ui.add_space(10.0);

        if ui.button("New Calculation").clicked() {
            // Reset state and go back to the main calculator view
            self.input_string.clear();
            self.animation_state = None;
            self.current_view = AppView::Calculator;
        }
    }
}