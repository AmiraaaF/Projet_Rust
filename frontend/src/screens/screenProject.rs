use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke, Color32};
use crate::state::{AppState, Screen};
use crate::screens::screenDashboard::{sidebar_item, sidebar_item_with_badge};
use crate::screens::screenTasks::{task_form_without_project, task_card};
use chrono;

pub fn projects_screen(ctx: &egui::Context, state: &mut AppState) {
    // Charger les projets si la liste est vide et qu'on est connecté
    if state.projects.is_empty() && state.token.is_some() && !state.is_loading {
        state.load_projects_sync();
    }
    
    let bg          = state.theme.background;
    let sidebar_bg  = state.theme.sidebar;
    let fg          = state.theme.foreground;
    let muted       = state.theme.muted_foreground;
    let border      = state.theme.border;
    let card        = state.theme.card;
    let primary     = state.theme.primary;
    let primary_fg  = state.theme.primary_foreground;
    let destructive = state.theme.destructive;
    let chart_2     = state.theme.chart_2;
    let destructive_fg = state.theme.destructive_foreground;

    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📁 Projects").color(fg).size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let logout_btn = egui::Button::new(
                        RichText::new("🔓 Logout").color(destructive_fg).size(13.0)
                    ).fill(destructive);
                    if ui.add(logout_btn).clicked() {
                        state.logout();
                    }
                    if let Some(user) = &state.current_user.clone() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("👤 {}", user.name)).color(muted).size(13.0));
                    }
                });
            });
        });

    egui::SidePanel::left("sidebar")
        .show_separator_line(false)
        .min_width(180.0)
        .max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);

            if sidebar_item(ui, "📊 Dashboard", false, fg, primary) {
                state.go_to(Screen::Dashboard);
            }
            ui.add_space(4.0);
            sidebar_item(ui, "📁 Projects", true, fg, primary);
            ui.add_space(4.0);
            if sidebar_item(ui, "✅ Tasks", false, fg, primary) { state.go_to(Screen::Tasks); }
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing", false, fg, primary) {
                state.go_to(Screen::Billing);
            }
            ui.add_space(4.0);
            if sidebar_item_with_badge(ui, "🔔 Notifications", false, fg, primary, state.notif_state.unread_count) {
                state.go_to(Screen::Notifications);
            }
        });

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg).inner_margin(Margin::same(24.0)))
        .show(ctx, |ui| {
            ui.label(RichText::new("New project").color(fg).size(16.0).strong());
            ui.add_space(12.0);

            Frame::none()
                .fill(card)
                .stroke(Stroke::new(1.0, border))
                .inner_margin(Margin::same(16.0))
                .rounding(Rounding::same(8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Project name").color(fg).size(13.0));
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut state.project_name_input)
                            .hint_text("e.g. Client website X")
                            .desired_width(280.0)
                    );
                    ui.add_space(10.0);

                    ui.label(RichText::new("Description").color(fg).size(13.0));
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut state.project_description_input)
                            .hint_text("Project description...")
                            .desired_width(280.0)
                            .desired_rows(3)
                    );
                    ui.add_space(12.0);

                    let create_clicked = ui.add(
                        egui::Button::new(
                            RichText::new("✅ Create project").color(primary_fg).size(13.0)
                        )
                        .fill(primary)
                        .min_size(egui::vec2(160.0, 32.0))
                    ).clicked();

                    if let Some(error) = &state.error_message.clone() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("⚠ {}", error)).color(state.theme.destructive).size(12.0));
                    }
                    if let Some(success) = &state.success_message.clone() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("✅ {}", success)).color(chart_2).size(12.0));
                    }

                    if create_clicked {
                        if state.project_name_input.trim().is_empty() {
                            state.success_message = None;
                            state.error_message = Some("Please enter a project name".to_string());
                        } else {
                            // Use centralized AppState method which handles token and API call
                            state.is_loading = true;
                            // clone inputs to avoid borrowing `state` immutably while calling a mutable method
                            let name = state.project_name_input.clone();
                            let description_owned = if state.project_description_input.trim().is_empty() {
                                None
                            } else {
                                Some(state.project_description_input.clone())
                            };

                            eprintln!("DEBUG: create project clicked, name='{}'", name);

                            match state.create_project_sync(&name, description_owned.as_deref()) {
                                Ok(()) => {
                                    state.error_message = None;
                                    state.success_message = Some("Project created successfully!".to_string());
                                    state.project_name_input.clear();
                                    state.project_description_input.clear();
                                    eprintln!("DEBUG: project creation succeeded");
                                }
                                Err(err) => {
                                    state.success_message = None;
                                    state.error_message = Some(format!("Failed to create project: {}", err));
                                    eprintln!("DEBUG: project creation failed: {}", err);
                                }
                            }

                            state.is_loading = false;
                        }
                    }
                });

            ui.add_space(24.0);

            ui.label(RichText::new("Your projects").color(fg).size(16.0).strong());
            ui.add_space(12.0);

            if state.projects.is_empty() {
                ui.label(RichText::new("No projects yet.").color(muted).size(13.0));
            } else {
                let projects_clone = state.projects.clone();
                for project in &projects_clone {
                    Frame::none()
                        .fill(card)
                        .stroke(Stroke::new(1.0, border))
                        .inner_margin(Margin::symmetric(16.0, 12.0))
                        .rounding(Rounding::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("📁 {}", project.name)).color(fg).size(14.0));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let btn = egui::Button::new(
                                        RichText::new("Open").color(primary_fg).size(12.0)
                                    ).fill(primary);
                                    if ui.add(btn).clicked() {
                                        // Set current project and preload its tasks before navigating
                                        state.current_project = Some(project.clone());
                                        // load tasks for the selected project (synchronous blocking call)
                                        state.load_tasks_sync(Some(&project.id.to_string()));
                                        state.go_to(Screen::ProjectDetail);
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            }
        });
}

pub fn project_detail_screen(ctx: &egui::Context, state: &mut AppState) {
    let bg          = state.theme.background;
    let sidebar_bg  = state.theme.sidebar;
    let fg          = state.theme.foreground;
    let muted       = state.theme.muted_foreground;
    let border      = state.theme.border;
    let card        = state.theme.card;
    let primary     = state.theme.primary;
    let primary_fg  = state.theme.primary_foreground;
    let destructive = state.theme.destructive;
    let destructive_fg = state.theme.destructive_foreground;
    let sidebar_primary = state.theme.sidebar_primary;

    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(project) = &state.current_project.clone() {
                    ui.label(RichText::new(format!("📁 {}", project.name)).color(fg).size(18.0).strong());
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(user) = &state.current_user.clone() {
                        ui.label(RichText::new(format!("👤 {}", user.name)).color(muted).size(13.0));
                    }
                    ui.add_space(8.0);
                    let logout_btn = egui::Button::new(
                        RichText::new("🔓 Logout").color(destructive_fg).size(13.0)
                    ).fill(destructive);
                    if ui.add(logout_btn).clicked() {
                        state.logout();
                    }
                });
            });
        });

    egui::SidePanel::left("sidebar")
        .show_separator_line(false)
        .min_width(180.0)
        .max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);

            if sidebar_item(ui, "📊 Dashboard", false, fg, primary) {
                state.go_to(Screen::Dashboard);
            }
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projects", true, fg, primary) {
                state.current_project = None;
                state.go_to(Screen::Projects);
            }
            ui.add_space(4.0);
            if sidebar_item(ui, "✅ Tasks", false, fg, primary) { state.go_to(Screen::Tasks); }
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing", false, fg, primary) {
                state.go_to(Screen::Billing);
            }
            ui.add_space(4.0);
            if sidebar_item_with_badge(ui, "🔔 Notifications", false, fg, primary, state.notif_state.unread_count) {
                state.go_to(Screen::Notifications);
            }
        });

    egui::CentralPanel::default()
        .frame(egui::Frame::none().fill(bg).inner_margin(egui::Margin::same(20.0)))
        .show(ctx, |ui| {
            if let Some(project) = &state.current_project.clone() {
                // Header du projet
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new(&project.name).color(fg).size(24.0).strong());
                        ui.label(egui::RichText::new("Project kanban board").color(state.theme.muted_foreground));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("➕ Add Task").strong()).clicked() {
                            // ouvrir le formulaire d'ajout
                            state.show_add_task_form = true;
                        }
                    });
                });

                // petit formulaire modal/inline pour ajouter une tâche (visible si state.show_add_task_form == true)
                if state.show_add_task_form {
                    if let Some(project) = &state.current_project.clone() {
                        // Charger les membres du projet si pas encore chargés
                        if state.members_of_current_project.is_empty() {
                            state.load_project_members_sync(&project.id.to_string());
                        }
                        
                        task_form_without_project(
                            ui, state,
                            project.id.to_string(),
                            card, fg, muted, border,
                            primary, primary_fg,
                            primary, primary_fg,
                        );
                    }
                }

                ui.add_space(12.0);

                // --- LISTE DES TÂCHES DU PROJET ---
                if state.current_tasks.is_empty() {
                    ui.label(RichText::new("Aucune tâche pour l'instant.").color(muted).size(14.0));
                } else {
                    let tasks_clone = state.current_tasks.clone();
                    let current_user_id = state.current_user.as_ref().map(|u| u.id.to_string());
                    let green = state.theme.chart_2;
                    let amber = state.theme.chart_3;
                    let destructive = state.theme.destructive;

                    for task in &tasks_clone {
                        let (clicked_done, clicked_delete) = task_card(
                            ui, task, &current_user_id,
                            card, border, fg, muted,
                            green, amber, destructive, sidebar_primary,
                        );

                        if clicked_done {
                            if let Some(pos) = state.current_tasks.iter().position(|t| t.id == task.id) {
                                state.current_tasks[pos].status = "done".to_string();
                                state.current_tasks[pos].updated_at = chrono::Utc::now();
                            }
                        }

                        if clicked_delete {
                            state.current_tasks.retain(|t| t.id != task.id);
                        }

                        ui.add_space(8.0);
                    }
                }
            }
        });
}
