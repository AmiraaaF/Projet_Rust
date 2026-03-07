use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke, ScrollArea, Color32};
use crate::state::{AppState, Screen};
use shared::models::Task;

pub fn tasks_screen(ctx: &egui::Context, state: &mut AppState) {
    state.load_tasks_sync();

    // ── Couleurs depuis le thème ─────────────────────────────────────────────
    let bg             = state.theme.background;
    let sidebar_bg     = state.theme.sidebar;
    let fg             = state.theme.foreground;
    let muted          = state.theme.muted_foreground;
    let border         = state.theme.border;
    let card           = state.theme.card;
    let primary        = state.theme.primary;
    let primary_fg     = state.theme.primary_foreground;
    let destructive    = state.theme.destructive;
    let destructive_fg = state.theme.destructive_foreground;
    let sidebar_primary    = state.theme.sidebar_primary;           
    let sidebar_primary_fg = state.theme.sidebar_primary_foreground; 
    let violet         = state.theme.chart_1;  
    let green          = state.theme.chart_2;  
    let amber          = state.theme.chart_3;  
    let grey           = state.theme.secondary; 

    // ── Calcul des stats depuis les tâches déjà chargées ────────────────────
    let total        = state.current_tasks.len();
    let todo_count   = state.current_tasks.iter().filter(|t| t.status == "todo").count();
    let inprog_count = state.current_tasks.iter().filter(|t| t.status == "in_progress").count();
    let done_count   = state.current_tasks.iter().filter(|t| t.status == "done").count();

    // ── TOP BAR ──────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("tasks_top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✅ Tasks").color(fg).size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let logout_btn = egui::Button::new(
                        RichText::new("🔓 Logout").color(destructive_fg).size(13.0),
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

    // ── SIDEBAR ───────────────────────────────────────────────────────────────
    egui::SidePanel::left("tasks_sidebar")
        .show_separator_line(false)
        .min_width(180.0).max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);
            if sidebar_item(ui, "📊 Dashboard", false, fg, primary) { state.go_to(Screen::Dashboard); }
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projects", false, fg, primary) { state.go_to(Screen::Projects); }
            ui.add_space(4.0);
            sidebar_item(ui, "✅ Tasks", true, sidebar_primary_fg, primary);
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing", false, fg, primary) { state.go_to(Screen::Billing); }
            ui.add_space(4.0);
        });

    // ── CENTRAL PANEL ─────────────────────────────────────────────────────────
    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg).inner_margin(Margin { left: 32.0, right: 56.0, top: 0.0, bottom: 0.0 }))
        .show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                ui.add_space(24.0);
                let content_width = ui.available_width();
                ui.vertical(|ui| {
                    ui.set_max_width(content_width);

                    // ── Titre + bouton "+ New Task" sur la même ligne ─────
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Tasks").color(fg).size(22.0).strong());
                            ui.add_space(4.0);
                            ui.label(RichText::new("Vue d'ensemble de vos tâches").color(muted).size(13.0));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn = egui::Button::new(
                                RichText::new("+ New Task").color(sidebar_primary_fg).size(13.0)
                            ).fill(sidebar_primary);
                            if ui.add(btn).clicked() {
                                state.show_task_form = true;
                            }
                        });
                    });
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        stat_card(ui, "Total",       &total.to_string(),        card, fg, muted, border, violet);
                        ui.add_space(12.0);
                        stat_card(ui, "Todo",        &todo_count.to_string(),   card, fg, muted, border, grey);
                        ui.add_space(12.0);
                        stat_card(ui, "In Progress", &inprog_count.to_string(), card, fg, muted, border, amber);
                        ui.add_space(12.0);
                        stat_card(ui, "Done",        &done_count.to_string(),   card, fg, muted, border, green);
                    });

                    ui.add_space(24.0);

                    // ── FILTRES ───────────────────────────────────────────
                    ui.horizontal(|ui| {
              
                        let filter_btn = |ui: &mut egui::Ui, label: &str, is_active: bool| -> bool {
                            let color = if is_active { sidebar_primary } else { grey };
                            let fg = if is_active { sidebar_primary_fg } else { muted };
                            ui.add(egui::Button::new(RichText::new(label).color(fg).size(13.0)).fill(color)).clicked()
                        };

                        // "Mes tâches"
                        if filter_btn(ui, "Mes tâches", state.filter_my_tasks) {
                            state.filter_my_tasks = !state.filter_my_tasks;
                            state.filter_changed = true;
                        }
                        ui.add_space(8.0);

                        for (label, status) in [("Todo", "todo"), ("In Progress", "in_progress"), ("Done", "done")] {
                            let is_active = state.filter_status.as_deref() == Some(status);
                            if filter_btn(ui, label, is_active) {
                                state.filter_status = if is_active { None } else { Some(status.to_string()) };
                                state.filter_changed = true;
                            }
                            ui.add_space(8.0);
                        }
                    });

                    ui.add_space(32.0);

                    if state.show_task_form {
                        task_form(
                            ui, state,
                            card, fg, muted, border,
                            sidebar_primary, sidebar_primary_fg,
                            primary, primary_fg,
                        );
                        ui.add_space(24.0);
                    }

                    // ── Liste des tâches ───────────────────────────────────
                    ui.label(RichText::new("Liste des tâches").color(fg).size(18.0).strong());
                    ui.add_space(16.0);

                    let tasks_clone    = state.current_tasks.clone();
                    let projects_clone = state.projects.clone();
                    let token_clone    = state.token.clone();
                    let current_user_id = state.current_user.as_ref().map(|u| u.id.to_string());
                    let mut to_mark_done: Option<String> = None;
                    let mut to_delete:    Option<String> = None;

                    if tasks_clone.is_empty() {
                        ui.label(RichText::new("Aucune tâche pour l'instant.").color(muted).size(14.0));
                    } else {
                        for task in &tasks_clone {
                            let project_name = projects_clone
                                .iter()
                                .find(|p| p.id == task.project_id)
                                .map(|p| p.name.as_str())
                                .unwrap_or("—");

                            let (clicked_done, clicked_delete) = task_card(
                                ui, task, &current_user_id,
                                card, border, fg, muted,
                                green, amber, destructive, sidebar_primary,
                            );

                            if clicked_done   { to_mark_done = Some(task.id.to_string()); }
                            if clicked_delete { to_delete    = Some(task.id.to_string()); }

                            ui.add_space(8.0);
                        }
                    }
                    if let (Some(task_id), Some(token)) = (to_mark_done, token_clone.clone()) {
                        let _ = state.api_client.mark_task_done_sync(&task_id, &token);
                        state.tasks_loaded = false; // recharge la liste la prochaine frame
                    }
                    if let (Some(task_id), Some(token)) = (to_delete, token_clone) {
                        let _ = state.api_client.delete_task_sync(&task_id, &token);
                        state.tasks_loaded = false;
                    }
                });
            });
        });
}

//  FORMULAIRE DE CRÉATION DE TÂCHE
fn task_form(
    ui: &mut egui::Ui,
    state: &mut AppState,
    card: Color32,
    fg: Color32,
    muted: Color32,
    border: Color32,
    sidebar_primary: Color32,
    sidebar_primary_fg: Color32,
    _primary: Color32,
    _primary_fg: Color32,
) {
    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.5, sidebar_primary))
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());

            ui.label(RichText::new("Nouvelle tâche").color(fg).size(16.0).strong());
            ui.add_space(16.0);

            // ── Titre ─────────────────────────────────────────────────────
            ui.label(RichText::new("Titre *").color(muted).size(12.0));
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut state.task_title_input)
                    .hint_text("Nom de la tâche")
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(12.0);

            // ── Projet obligatoire ────────────────────────────────────────
            ui.label(RichText::new("Projet *").color(muted).size(12.0));
            ui.add_space(4.0);

            //menu derroulant qui affiche la liste des projets qui appartienne a l'utilisateur
            let projects_clone = state.projects.clone();
            let selected_project_name = projects_clone
                .iter()
                .find(|p| p.id.to_string() == state.task_project_id_input)
                .map(|p| p.name.as_str())
                .unwrap_or("-- Sélectionner un projet --"); 
            let old_project_id = state.task_project_id_input.clone();

            egui::ComboBox::from_id_source("task_project")
                .selected_text(selected_project_name)
                .width(300.0)
                .show_ui(ui, |ui| {
                    for project in &projects_clone {
                        ui.selectable_value(
                            &mut state.task_project_id_input,
                            project.id.to_string(),
                            &project.name,
                        );
                    }
                });

            ui.add_space(12.0);


            // ── Assignee : membres du projet sélectionné ──────────────────
            // Affiché seulement si un projet est sélectionné
            // if !state.task_project_id_input.is_empty() {
            //     ui.label(RichText::new("Assigné à").color(muted).size(12.0));
            //     ui.add_space(4.0);

            //     // Nom affiché dans la ComboBox : "Moi-même" si vide, sinon le nom du membre
            //     let members_clone = state.form_project_members.clone();
            //     let selected_assignee_name = if state.task_assignee_input.is_empty() {
            //         "Moi-même".to_string()
            //     } else {
            //         members_clone
            //             .iter()
            //             .find(|(id, _)| id == &state.task_assignee_input)
            //             .map(|(_, name)| name.clone())
            //             .unwrap_or_else(|| "Moi-même".to_string())
            //     };

            //     egui::ComboBox::from_id_source("task_assignee")
            //         .selected_text(&selected_assignee_name)
            //         .width(300.0)
            //         .show_ui(ui, |ui| {
            //             // Option par défaut : assigné à la personne connectée
            //             ui.selectable_value(
            //                 &mut state.task_assignee_input,
            //                 String::new(),
            //                 "Moi-même",
            //             );
            //             // Un item par membre du projet
            //             for (member_id, member_name) in &members_clone {
            //                 ui.selectable_value(
            //                     &mut state.task_assignee_input,
            //                     member_id.clone(),
            //                     member_name.as_str(),
            //                 );
            //             }
            //         });
            //     ui.add_space(12.0);
            // }

            // ── Statut + Priorité côte à côte ────────────────────────────
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Statut *").color(muted).size(12.0));
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_source("task_status")
                        .selected_text(&state.task_status_input)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.task_status_input, "todo".to_string(),        "todo");
                            ui.selectable_value(&mut state.task_status_input, "in_progress".to_string(), "in_progress");
                            ui.selectable_value(&mut state.task_status_input, "done".to_string(),        "done");
                        });
                });
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Priorité *").color(muted).size(12.0));
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_source("task_priority")
                        .selected_text(&state.task_priority_input)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.task_priority_input, "low".to_string(),    "low");
                            ui.selectable_value(&mut state.task_priority_input, "medium".to_string(), "medium");
                            ui.selectable_value(&mut state.task_priority_input, "high".to_string(),   "high");
                        });
                });
            });
            ui.add_space(12.0);

            // ── Description (optionnel) ───────────────────────────────────
            ui.label(RichText::new("Description (optionnel)").color(muted).size(12.0));
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::multiline(&mut state.task_description_input)
                    .hint_text("Décrivez la tâche...")
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(12.0);

            // ── Deadline (optionnel) ──────────────────────────────────────
            ui.label(RichText::new("Deadline (optionnel)").color(muted).size(12.0));
            ui.add_space(4.0);
            ui.add(
                egui::TextEdit::singleline(&mut state.task_deadline_input)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(200.0),
            );
            ui.add_space(20.0);

            // ── Boutons Créer / Annuler ───────────────────────────────────
            ui.horizontal(|ui| {
                let creer_btn = egui::Button::new(
                    RichText::new("Créer").color(sidebar_primary_fg).size(13.0)
                )
                .fill(sidebar_primary)
                .min_size(egui::vec2(100.0, 32.0));

                if ui.add(creer_btn).clicked() {
                    // Validation : le projet est obligatoire
                    if state.task_project_id_input.is_empty() {
                        state.error_message = Some("Veuillez sélectionner un projet".to_string());
                    } else if let Some(token) = state.token.clone() {
                        let description = if state.task_description_input.is_empty() {
                            None
                        } else {
                            Some(state.task_description_input.as_str())
                        };
                        let deadline = if state.task_deadline_input.is_empty() {
                            None
                        } else {
                            Some(state.task_deadline_input.as_str())
                        };
                        // Si assignee vide → on passe None (le backend assignera à l'utilisateur connecté)
                        let assignee_id = if state.task_assignee_input.is_empty() {
                            None
                        } else {
                            Some(state.task_assignee_input.as_str())
                        };

                        match state.api_client.create_task_on_service_sync(
                            &state.task_title_input.clone(),
                            description,
                            &state.task_status_input.clone(),
                            &state.task_priority_input.clone(),
                            assignee_id,
                            deadline,
                            Some(state.task_project_id_input.as_str()),
                            &token,
                        ) {
                            Ok(_) => {
                                state.clear_forms();
                                state.success_message = Some("Tâche créée avec succès".to_string());
                            }
                            Err(e) => {
                                state.error_message = Some(format!("Erreur: {}", e));
                            }
                        }
                    } else {
                        state.error_message = Some("Vous devez être connecté".to_string());
                    }
                }

                ui.add_space(8.0);

                let annuler_btn = egui::Button::new(
                    RichText::new("Annuler").color(fg).size(13.0)
                )
                .fill(Color32::from_rgb(68, 68, 68))
                .min_size(egui::vec2(100.0, 32.0));

                if ui.add(annuler_btn).clicked() {
                    state.clear_forms();
                }
            });

            if let Some(err) = &state.error_message.clone() {
                ui.add_space(8.0);
                ui.label(RichText::new(format!("⚠ {}", err)).color(Color32::from_rgb(239, 68, 68)).size(12.0));
            }
            if let Some(ok) = &state.success_message.clone() {
                ui.add_space(8.0);
                ui.label(RichText::new(format!("✅ {}", ok)).color(Color32::from_rgb(132, 204, 22)).size(12.0));
            }
        });
}

//  HELPERS
fn stat_card(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    card: Color32,
    _fg: Color32,
    muted: Color32,
    border: Color32,
    accent: Color32,
) {
    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(16.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_min_width(120.0);
            ui.label(RichText::new(label).color(muted).size(12.0));
            ui.add_space(4.0);
            ui.label(RichText::new(value).color(accent).size(24.0).strong());
        });
}

//  CARD D'UNE TÂCHE
fn task_card(
    ui: &mut egui::Ui,
    task: &Task,
    current_user_id: &Option<String>,
    card: Color32,
    border: Color32,
    fg: Color32,
    muted: Color32,
    green: Color32,
    amber: Color32,
    destructive: Color32,
    sidebar_primary: Color32,
) -> (bool, bool) {
    let is_done = task.status == "done";
    let mut clicked_done   = false;
    let mut clicked_delete = false;

    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(16.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());

            //Checkbox + Titre + Bouton supprimer
            ui.horizontal(|ui| {
                let mut done_state = is_done;
                if ui.checkbox(&mut done_state, "").clicked() && !is_done {
                    clicked_done = true;
                }
                let title_rich = if is_done {
                    RichText::new(&task.title).color(muted).size(14.0).strong().strikethrough()
                } else {
                    RichText::new(&task.title).color(fg).size(14.0).strong()
                };
                ui.label(title_rich);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let del = egui::Button::new(RichText::new("❌" ).color(muted).size(12.0))
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE);
                    if ui.add(del).clicked() {
                        clicked_delete = true;
                    }
                });
            });

            // Description
            if let Some(desc) = &task.description {
                ui.add_space(4.0);
                let affichage = if desc.len() > 120 {
                    format!("{}…", &desc[..120])
                } else {
                    desc.clone()
                };
                ui.label(RichText::new(&affichage).color(muted).size(12.0));
            }

            ui.add_space(10.0);

            //personne assingee a la tache
            ui.horizontal(|ui| {
                let assignee_display = match &task.assignee_name {
                    None => "—".to_string(),
                    Some(aid) => {
                        let aid_str = aid.to_string();
                        if current_user_id.as_deref() == Some(&aid_str) {
                            "Moi".to_string()
                        } else {
                            format!("{}", &aid_str)
                        }
                    }
                };
                Frame::none()
                    .fill(sidebar_primary)
                    .rounding(Rounding::same(10.0))
                    .inner_margin(Margin::symmetric(6.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&assignee_display).color(Color32::WHITE).size(11.0));
                    });

                if let Some(dl) = &task.deadline {
                    ui.add_space(16.0);
                    let date_str = dl.format("%Y-%m-%d").to_string();
                    ui.label(RichText::new(format!("📅 {}", date_str)).color(muted).size(12.0));
                }
            });

            ui.add_space(8.0);

            //Projet + Badges priorité + statut
            ui.horizontal(|ui| {
                
                ui.label(
                    RichText::new(format!(
                        "📁 {}",
                        task.project_name.as_deref().unwrap_or("-")
                    ))
                    .color(muted)
                    .size(12.0),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (status_label, status_color) = match task.status.as_str() {
                        "done"        => ("done",        green),
                        "in_progress" => ("in progress", amber),
                        _             => ("todo",        Color32::from_rgb(100, 100, 100)),
                    };
                    badge(ui, status_label, status_color);

                    ui.add_space(6.0);
                    let (prio_label, prio_color) = match task.priority.as_str() {
                        "high"   => ("high",   destructive),
                        "medium" => ("medium", amber),
                        _        => ("low",    green),
                    };
                    badge(ui, prio_label, prio_color);
                });
            });
        });

    (clicked_done, clicked_delete)
}

// Petit badge 
fn badge(ui: &mut egui::Ui, label: &str, color: Color32) {
    Frame::none()
        .fill(Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 35))
        .stroke(Stroke::new(2.0, color))
        .rounding(Rounding::same(4.0))
        .inner_margin(Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).color(color).size(11.0));
        });
}

fn sidebar_item(ui: &mut egui::Ui, label: &str, active: bool, fg: Color32, primary: Color32) -> bool {
    let fill = if active {
        Color32::from_rgba_unmultiplied(124, 58, 202, 60) // violet semi-transparent
    } else {
        Color32::TRANSPARENT
    };
    let stroke = if active {
        Stroke::new(1.5, Color32::from_rgb(124, 58, 202)) // violet opaque
    } else {
        Stroke::NONE
    };
    let btn = egui::Button::new(RichText::new(label).color(if active { primary } else { fg }).size(14.0))
        .fill(fill)
        .stroke(stroke)
        .min_size(egui::vec2(156.0, 32.0));
    ui.add(btn).clicked()
}
