use eframe::egui;
use crate::state::{AppState, Screen};
use shared::models::*;

pub fn projects_screen(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.current_screen = Screen::Dashboard;
            }
            ui.heading("📁 Projects");
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("Project Name:");
        ui.text_edit_singleline(&mut state.project_name_input);

        ui.label("Description:");
        ui.text_edit_multiline(&mut state.project_description_input);

        if ui.button("✅ Create Project").clicked() {
            if !state.project_name_input.is_empty() {
                state.success_message = Some("Project created successfully!".to_string());
                state.project_name_input.clear();
                state.project_description_input.clear();
            } else {
                state.error_message = Some("Please enter a project name".to_string());
            }
        }

        ui.separator();
        ui.label("Your Projects:");

        for project in &state.projects {
            ui.horizontal(|ui| {
                ui.label(format!("📁 {}", project.name));
                if ui.button("View").clicked() {
                    state.current_project = Some(project.clone());
                    state.current_screen = Screen::ProjectDetail;
                }
            });
        }

        if state.projects.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "No projects yet.");
        }
    });
}

pub fn project_detail_screen(ctx: &egui::Context, state: &mut AppState) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                state.current_screen = Screen::Projects;
                state.current_project = None;
                state.current_tasks.clear();
            }

            if let Some(project) = &state.current_project {
                ui.heading(format!("📁 {}", project.name));
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(project) = &state.current_project {
            ui.label(format!(
                "Description: {}",
                project.description.as_deref().unwrap_or("No description")
            ));
            ui.label(format!("Status: {}", project.status));
            ui.separator();

            ui.label("Task Title:");
            ui.text_edit_singleline(&mut state.task_title_input);

            if ui.button("➕ Add Task").clicked() {
                if !state.task_title_input.is_empty() {
                    state.success_message = Some("Task created!".to_string());
                    state.task_title_input.clear();
                } else {
                    state.error_message = Some("Enter a task title".to_string());
                }
            }

            ui.separator();

            for task in &state.current_tasks {
                ui.label(format!("✓ {} [{}]", task.title, task.status));
            }
        }
    });
}