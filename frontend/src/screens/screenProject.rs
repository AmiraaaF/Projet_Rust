use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke};
use crate::state::{AppState, Screen};

pub fn projects_screen(ctx: &egui::Context, state: &mut AppState) {
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
            if sidebar_item(ui, "💳 Billing", false, fg, primary) {
                state.go_to(Screen::Billing);
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
                        ui.label(RichText::new(format!("✓ {}", success)).color(chart_2).size(12.0));
                    }

                    if create_clicked {
                        if !state.project_name_input.is_empty() {
                            state.error_message = None;
                            state.success_message = Some("Project created successfully!".to_string());
                            state.project_name_input.clear();
                            state.project_description_input.clear();
                        } else {
                            state.success_message = None;
                            state.error_message = Some("Please enter a project name".to_string());
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
                                        state.current_project = Some(project.clone());
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
    let chart_2     = state.theme.chart_2;

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
            if sidebar_item(ui, "💳 Billing", false, fg, primary) {
                state.go_to(Screen::Billing);
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
                            // Action ajouter tâche
                        }
                    });
                });

                ui.add_space(20.0);

                // --- LE KANBAN ---
                // On divise l'espace en 3 colonnes égales
                ui.columns(3, |columns| {
                    render_kanban_column(&mut columns[0], "Todo", "Todo", state);
                    render_kanban_column(&mut columns[1], "In Progress", "InProgress", state);
                    render_kanban_column(&mut columns[2], "Done", "Done", state);
                });
            }
        });
}

// Fonction pour dessiner une colonne (Todo, InProgress, etc.)
fn render_kanban_column(ui: &mut egui::Ui, title: &str, status_filter: &str, state: &mut AppState) {
    ui.vertical(|ui| {
        // En-tête de colonne avec le badge du nombre
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).strong().size(16.0).color(state.theme.foreground));
            let count = state.current_tasks.iter().filter(|t| t.status == status_filter).count();
            
            // Petit badge gris pour le compte
            egui::Frame::none()
                .fill(egui::Color32::from_gray(60))
                .rounding(10.0)
                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(count.to_string()).size(10.0).color(egui::Color32::WHITE));
                });
        });

        ui.add_space(12.0);

        // Zone scrollable pour les cartes
        egui::ScrollArea::vertical()
            .id_source(title)
            .show(ui, |ui| {
                for task in &state.current_tasks {
                    if task.status == status_filter {
                        draw_task_card(ui, task, state);
                        ui.add_space(8.0);
                    }
                }
            });
    });
}

// Fonction pour dessiner une "Card" de tâche comme sur l'image
fn draw_task_card(ui: &mut egui::Ui, task: &crate::state::Task, state: &AppState) {
    let priority_color = match task.priority.as_str() {
        "high" => egui::Color32::from_rgb(239, 68, 68),   // Rouge
        "medium" => egui::Color32::from_rgb(245, 158, 11), // Orange
        _ => egui::Color32::from_rgb(34, 197, 94),        // Vert
    };

    egui::Frame::none()
        .fill(state.theme.card) // Gris foncé #252525
        .rounding(6.0)
        .stroke(egui::Stroke::new(1.0, state.theme.border))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            
            ui.vertical(|ui| {
                // Titre de la tâche
                ui.label(egui::RichText::new(&task.title).strong().color(state.theme.foreground));
                
                // Tags
                ui.horizontal(|ui| {
                    for tag in &task.tags {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgba_unmultiplied(100, 100, 255, 30))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(4.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(tag).size(9.0).color(egui::Color32::from_rgb(150, 150, 255)));
                            });
                    }
                });

                ui.add_space(12.0);

                // Footer : Avatar + Assignee + Date
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("👤 {}", task.assignee)).size(11.0).color(state.theme.muted_foreground));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(&task.due_date).size(10.0).color(state.theme.muted_foreground));
                    });
                });

                ui.add_space(4.0);

                // Badge de priorité en bas
                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(priority_color.r(), priority_color.g(), priority_color.b(), 20))
                    .rounding(4.0)
                    .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&task.priority).color(priority_color).size(10.0).strong());
                    });
            });
        });
}
fn sidebar_item(ui: &mut egui::Ui, label: &str, active: bool, fg: egui::Color32, primary: egui::Color32) -> bool {
    let color = if active { primary } else { fg };
    let btn = egui::Button::new(RichText::new(label).color(color).size(14.0))
        .fill(egui::Color32::TRANSPARENT)
        .min_size(egui::vec2(156.0, 32.0));
    ui.add(btn).clicked()
}