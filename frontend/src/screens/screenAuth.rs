use eframe::egui;
use crate::state::{AppState, Screen};
use shared::models::*;

pub fn login_screen(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("🔐 Mini-SaaS Login");
        ui.separator();

        ui.label("Email:");
        ui.text_edit_singleline(&mut state.email_input);

        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut state.password_input).password(true));

        ui.horizontal(|ui| {
            if ui.button("Login").clicked() {
                let email = state.email_input.clone();
                let password = state.password_input.clone();

                if !email.is_empty() && !password.is_empty() {
                    state.success_message = Some("Login successful!".to_string());
                    state.current_screen = Screen::Dashboard;
                    state.current_user = Some(UserPublic {
                        id: uuid::Uuid::new_v4(),
                        email,
                        name: "User".to_string(),
                        role: "user".to_string(),
                        created_at: chrono::Utc::now(),
                    });
                } else {
                    state.error_message = Some("Please fill in all fields".to_string());
                }
            }

            if ui.button("Register").clicked() {
                state.current_screen = Screen::Register;
                state.clear_forms();
            }
        });

        if let Some(error) = &state.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }
        if let Some(success) = &state.success_message {
            ui.colored_label(egui::Color32::GREEN, success);
        }
    });
}

pub fn register_screen(ctx: &egui::Context, state: &mut AppState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("📝 Register");
        ui.separator();

        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name_input);

        ui.label("Email:");
        ui.text_edit_singleline(&mut state.email_input);

        ui.label("Password:");
        ui.add(egui::TextEdit::singleline(&mut state.password_input).password(true));

        ui.horizontal(|ui| {
            if ui.button("Register").clicked() {
                if !state.name_input.is_empty()
                    && !state.email_input.is_empty()
                    && !state.password_input.is_empty()
                {
                    state.success_message =
                        Some("Registration successful! Please login.".to_string());
                    state.current_screen = Screen::Login;
                    state.clear_forms();
                } else {
                    state.error_message = Some("Please fill in all fields".to_string());
                }
            }

            if ui.button("Back to Login").clicked() {
                state.current_screen = Screen::Login;
                state.clear_forms();
            }
        });

        if let Some(error) = &state.error_message {
            ui.colored_label(egui::Color32::RED, error);
        }
    });
}


