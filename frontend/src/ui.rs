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