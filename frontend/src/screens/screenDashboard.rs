use eframe::egui;
use crate::state::{AppState, Screen};
use shared::models::*;

pub fn dashboard_screen(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("📊 Dashboard");
            if let Some(user) = &state.current_user {
                ui.label(format!("Welcome, {}!", user.name));
            }
            if ui.button("🔓 Logout").clicked() {
                state.logout();
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Projects Overview");
        ui.separator();

        if ui.button("📁 View All Projects").clicked() {
            state.current_screen = Screen::Projects;
        }

        ui.label(format!("Total Projects: {}", state.projects.len()));
        ui.label(format!("Total Tasks: {}", state.current_tasks.len()));

        ui.separator();
        ui.label("Recent Projects:");

        for project in state.projects.iter().take(5) {
            if ui.button(format!("📁 {}", project.name)).clicked() {
                state.current_project = Some(project.clone());
                state.current_screen = Screen::ProjectDetail;
            }
        }
    });
}