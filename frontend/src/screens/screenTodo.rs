use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, ScrollArea, Stroke, Vec2};
use crate::state::{AppState, Screen, TodoFilter, TodoItem, TodoPriority, TodoStatus};
use crate::screens::screenDashboard::{sidebar_item, sidebar_item_with_badge};

pub fn todo_screen(ctx: &egui::Context, state: &mut AppState) {
    state.poll_notifications_sync();
    if !state.todo_loaded {
        state.load_personal_tasks_sync();
    }
    ctx.request_repaint();

    let bg             = state.theme.background;
    let sidebar_bg     = state.theme.sidebar;
    let fg             = state.theme.foreground;
    let muted          = state.theme.muted_foreground;
    let border         = state.theme.border;
    let primary        = state.theme.primary;
    let primary_fg     = state.theme.primary_foreground;
    let destructive    = state.theme.destructive;
    let destructive_fg = state.theme.destructive_foreground;
    let card           = state.theme.card;
    let chart_2        = state.theme.chart_2;
    let chart_3        = state.theme.chart_3;
    let secondary      = state.theme.secondary;
    let secondary_fg   = state.theme.secondary_foreground;

    // ── TOP BAR ──────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("todo_top")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("✅ To-Do List").color(fg).size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(
                        RichText::new("🔓 Logout").color(destructive_fg).size(13.0)
                    ).fill(destructive)).clicked() {
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
    egui::SidePanel::left("todo_sidebar")
        .show_separator_line(false)
        .min_width(180.0).max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);
            if sidebar_item(ui, "📊 Dashboard", false, fg, primary) { state.go_to(Screen::Dashboard); }
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projects",     false, fg, primary) { state.go_to(Screen::Projects); }
            ui.add_space(4.0);
            if sidebar_item(ui, "✅ Tasks", false, fg, primary) { state.go_to(Screen::Tasks); }
            ui.add_space(4.0);
            if sidebar_item(ui, "✅ To-Do",        false, fg, primary) { state.go_to(Screen::Todo); }
            ui.add_space(4.0);
            if sidebar_item(ui, "📅 Calendar",     false, fg, primary) { state.go_to(Screen::Calendar); }
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing", false, fg, primary) { state.go_to(Screen::Billing); }
            ui.add_space(4.0);
            if sidebar_item_with_badge(ui, "🔔 Notifications", false, fg, primary, state.notif_state.unread_count) {
                state.go_to(Screen::Notifications);
            }
            ui.add_space(4.0);
            if sidebar_item(ui, "👤 Profile",      false, fg, primary) { state.go_to(Screen::Profile); }
        });

    // ── CENTRAL PANEL ─────────────────────────────────────────────────────────
    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg).inner_margin(Margin { left: 32.0, right: 32.0, top: 0.0, bottom: 0.0 }))
        .show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                ui.add_space(24.0);
                let content_width = ui.available_width();
                ui.vertical(|ui| {
                    ui.set_max_width(content_width);

                    // ── Header row ────────────────────────────────────────
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("My Tasks").color(fg).size(22.0).strong());
                            ui.add_space(2.0);
                            ui.label(RichText::new("Manage your personal to-do list and track progress")
                                .color(muted).size(13.0));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if !state.todo_state.show_form {
                                if ui.add(egui::Button::new(
                                    RichText::new("＋ New Task").color(primary_fg).size(13.0)
                                ).fill(primary).min_size(Vec2::new(110.0, 34.0))).clicked() {
                                    state.todo_state.show_form = true;
                                    state.todo_state.editing_id = None;
                                }
                            }
                        });
                    });
                    ui.add_space(20.0);

                    // ── Stats row ─────────────────────────────────────────
                    stats_row(ui, state, content_width, card, fg, muted, border, chart_2, chart_3);
                    ui.add_space(20.0);

                    // ── Task Creation / Edit Form ──────────────────────────
                    if state.todo_state.show_form {
                        task_form(ui, state, content_width, card, fg, muted, border,
                                  primary, primary_fg, secondary, secondary_fg,
                                  destructive, destructive_fg, chart_2, chart_3);
                        ui.add_space(20.0);
                    }

                    // ── Filter tabs ───────────────────────────────────────
                    filter_tabs(ui, state, fg, primary, muted, border);
                    ui.add_space(16.0);

                    // ── Task list ─────────────────────────────────────────
                    let items = filtered_items(state);
                    if items.is_empty() {
                        empty_state(ui, &state.todo_state.filter.clone(), content_width, card, border, fg, muted);
                    } else {
                        let n = items.len();
                        let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
                        for (idx, item) in items.iter().enumerate() {
                            task_row(ui, ctx, item, state, content_width,
                                     card, fg, muted, border,
                                     primary, primary_fg,
                                     destructive, destructive_fg,
                                     chart_2, chart_3, secondary, secondary_fg);
                            if idx < n - 1 { ui.add_space(6.0); }
                        }
                    }

                    ui.add_space(24.0);
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  STATS ROW
// ─────────────────────────────────────────────────────────────────────────────

fn stats_row(
    ui: &mut egui::Ui, state: &AppState, content_w: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    chart_2: Color32, chart_3: Color32,
) {
    let total       = state.todo_state.items.len();
    let todo_count  = state.todo_state.items.iter().filter(|i| i.status == TodoStatus::Todo).count();
    let wip_count   = state.todo_state.items.iter().filter(|i| i.status == TodoStatus::InProgress).count();
    let done_count  = state.todo_state.items.iter().filter(|i| i.status == TodoStatus::Done).count();
    let high_count  = state.todo_state.items.iter().filter(|i| i.priority == TodoPriority::High).count();
    let today       = chrono::Local::now().format("%Y-%m-%d").to_string();
    let overdue     = state.todo_state.items.iter().filter(|i| {
        i.status != TodoStatus::Done &&
        i.deadline.as_ref().map(|d| d.as_str() < today.as_str()).unwrap_or(false)
    }).count();

    let num_cols = if content_w > 900.0 { 5 } else if content_w > 600.0 { 3 } else { 2 };

    ui.columns(num_cols, |cols| {
        let stats = [
            ("📋 Total",      total.to_string(),      fg),
            ("⬜ To Do",       todo_count.to_string(), muted),
            ("🔄 In Progress", wip_count.to_string(),  chart_3),
            ("✅ Done",        done_count.to_string(), chart_2),
            ("🔴 High Prio",   high_count.to_string(), Color32::from_rgb(239, 68, 68)),
        ];
        for (idx, (label, value, val_color)) in stats.iter().enumerate() {
            if idx < num_cols {
                mini_stat_card(&mut cols[idx], label, value, *val_color, card, fg, muted, border);
            }
        }
    });

    if overdue > 0 {
        ui.add_space(8.0);
        Frame::none()
            .fill(Color32::from_rgb(60, 25, 25))
            .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
            .inner_margin(Margin::symmetric(14.0, 8.0))
            .rounding(Rounding::same(8.0))
            .show(ui, |ui| {
                ui.label(RichText::new(format!(
                    "⚠ {} task{} overdue — check deadlines",
                    overdue,
                    if overdue > 1 { "s are" } else { " is" }
                )).color(Color32::from_rgb(239, 68, 68)).size(12.0));
            });
    }
}

fn mini_stat_card(
    ui: &mut egui::Ui,
    label: &str, value: &str, val_color: Color32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
) {
    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(10.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).color(muted).size(11.0));
            ui.add_space(3.0);
            ui.label(RichText::new(value).color(val_color).size(22.0).strong());
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  FILTER TABS
// ─────────────────────────────────────────────────────────────────────────────

fn filter_tabs(
    ui: &mut egui::Ui, state: &mut AppState,
    fg: Color32, primary: Color32, muted: Color32, border: Color32,
) {
    ui.horizontal_wrapped(|ui| {
        for tab in &[
            TodoFilter::All,
            TodoFilter::Active,
            TodoFilter::InProgress,
            TodoFilter::Done,
            TodoFilter::HighPriority,
            TodoFilter::HasDeadline,
        ] {
            let is_active = &state.todo_state.filter == tab;
            let (fill, text_color, stroke_color) = if is_active {
                (Color32::from_rgb(124, 58, 202), Color32::WHITE, Color32::from_rgb(124, 58, 202))
            } else {
                (Color32::TRANSPARENT, muted, border)
            };

            if ui.add(
                egui::Button::new(RichText::new(tab.label()).color(text_color).size(12.0))
                    .fill(fill)
                    .stroke(Stroke::new(1.0, stroke_color))
                    .min_size(Vec2::new(70.0, 28.0))
            ).clicked() {
                state.todo_state.filter = tab.clone();
            }
            ui.add_space(4.0);
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
//  TASK FORM (Create / Edit)
// ─────────────────────────────────────────────────────────────────────────────

fn task_form(
    ui: &mut egui::Ui,
    state: &mut AppState,
    content_width: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    primary: Color32, primary_fg: Color32,
    secondary: Color32, secondary_fg: Color32,
    destructive: Color32, destructive_fg: Color32,
    chart_2: Color32, chart_3: Color32,
) {
    let is_editing = state.todo_state.editing_id.is_some();
    let title_str  = if is_editing { "✏ Edit Task" } else { "＋ New Task" };

    Frame::none()
        .fill(Color32::from_rgb(42, 42, 55))
        .stroke(Stroke::new(1.5, Color32::from_rgb(124, 58, 202)))
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);

            ui.horizontal(|ui| {
                ui.label(RichText::new(title_str).color(fg).size(16.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(RichText::new("✕").color(muted).size(14.0))
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(24.0, 24.0))).clicked() {
                        state.cancel_todo_form();
                    }
                });
            });
            ui.add_space(16.0);

            // Title
            ui.label(RichText::new("Title *").color(fg).size(13.0));
            ui.add_space(4.0);
            ui.add(egui::TextEdit::singleline(&mut state.todo_state.form_title)
                .hint_text("e.g. Review pull request")
                .desired_width(content_width - 40.0));
            ui.add_space(12.0);

            // Description
            ui.label(RichText::new("Description").color(fg).size(13.0));
            ui.add_space(4.0);
            ui.add(egui::TextEdit::multiline(&mut state.todo_state.form_description)
                .hint_text("Optional details...")
                .desired_width(content_width - 40.0)
                .desired_rows(2));
            ui.add_space(12.0);

            // Priority + Deadline + Project on one row
            ui.columns(3, |cols| {
                // Priority
                cols[0].label(RichText::new("Priority").color(fg).size(13.0));
                cols[0].add_space(4.0);
                let prio_label = format!(
                    "{} {}",
                    state.todo_state.form_priority.emoji(),
                    state.todo_state.form_priority.label()
                );
                if cols[0].add(egui::Button::new(
                    RichText::new(&prio_label).color(fg).size(12.0))
                    .fill(Color32::from_rgb(55, 55, 68))
                    .stroke(Stroke::new(1.0, border))
                    .min_size(Vec2::new(120.0, 30.0))).clicked()
                {
                    let next = state.todo_state.form_priority.next();
                    state.todo_state.form_priority = next;
                }

                // Deadline
                cols[1].label(RichText::new("Deadline (YYYY-MM-DD)").color(fg).size(13.0));
                cols[1].add_space(4.0);
                cols[1].add(egui::TextEdit::singleline(&mut state.todo_state.form_deadline)
                    .hint_text("2026-03-31")
                    .desired_width(140.0));

                // Project tag
                cols[2].label(RichText::new("Project / Tag").color(fg).size(13.0));
                cols[2].add_space(4.0);
                cols[2].add(egui::TextEdit::singleline(&mut state.todo_state.form_project)
                    .hint_text("e.g. Website Redesign")
                    .desired_width(140.0));
            });

            ui.add_space(16.0);

            // Error
            if let Some(err) = &state.todo_state.form_error.clone() {
                ui.label(RichText::new(format!("⚠ {}", err))
                    .color(destructive_fg).size(12.0));
                ui.add_space(8.0);
            }

            // Buttons
            ui.horizontal(|ui| {
                let btn_label = if is_editing { "Save Changes" } else { "Create Task" };
                if ui.add(egui::Button::new(
                    RichText::new(btn_label).color(primary_fg).size(13.0)
                ).fill(primary).min_size(Vec2::new(120.0, 32.0))).clicked() {
                    if is_editing {
                        state.save_edit_todo_sync();
                    } else {
                        state.add_todo_sync();
                    }
                }
                ui.add_space(8.0);
                if ui.add(egui::Button::new(
                    RichText::new("Cancel").color(fg).size(13.0)
                ).fill(secondary).min_size(Vec2::new(80.0, 32.0))).clicked() {
                    state.cancel_todo_form();
                }
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  TASK ROW
// ─────────────────────────────────────────────────────────────────────────────

fn task_row(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    item: &TodoItem,
    state: &mut AppState,
    content_width: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    primary: Color32, primary_fg: Color32,
    destructive: Color32, destructive_fg: Color32,
    chart_2: Color32, chart_3: Color32,
    secondary: Color32, secondary_fg: Color32,
) {
    let id    = item.id.clone();
    let done  = item.status == TodoStatus::Done;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let is_overdue = !done && item.deadline.as_ref().map(|d| d.as_str() < today.as_str()).unwrap_or(false);

    let row_fill   = if done { Color32::from_rgb(35, 45, 35) } else if is_overdue { Color32::from_rgb(45, 30, 30) } else { card };
    let row_border = if is_overdue { Color32::from_rgb(239, 68, 68) }
                     else if done  { chart_2 }
                     else          { border };

    Frame::none()
        .fill(row_fill)
        .stroke(Stroke::new(1.0, row_border))
        .inner_margin(Margin::symmetric(14.0, 12.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);
            ui.horizontal(|ui| {
                // Checkbox
                let mut is_done = done;
                if ui.checkbox(&mut is_done, "").changed() {
                    let id2 = id.clone();
                    state.toggle_todo_done_sync(&id2);
                    ctx.request_repaint();
                }
                ui.add_space(4.0);

                // Priority dot
                let prio_color = match item.priority {
                    TodoPriority::High   => Color32::from_rgb(239, 68, 68),
                    TodoPriority::Medium => Color32::from_rgb(245, 158, 11),
                    TodoPriority::Low    => Color32::from_rgb(132, 204, 22),
                };
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, prio_color);
                ui.add_space(8.0);

                // Content
                let text_width = (content_width - 220.0).max(100.0);
                ui.vertical(|ui| {
                    ui.set_max_width(text_width);

                    // Title
                    let title_color = if done { muted } else { fg };
                    let title_text  = if done {
                        RichText::new(&item.title).color(title_color).size(13.5).strikethrough()
                    } else {
                        RichText::new(&item.title).color(title_color).size(13.5).strong()
                    };
                    ui.label(title_text);

                    if !item.description.is_empty() {
                        ui.add_space(2.0);
                        ui.label(RichText::new(&item.description).color(muted).size(11.5));
                    }

                    ui.add_space(4.0);
                    ui.horizontal_wrapped(|ui| {
                        // Status badge
                        status_badge(ui, &item.status, chart_2, chart_3, muted);
                        ui.add_space(4.0);

                        // Deadline badge
                        if let Some(deadline) = &item.deadline {
                            let (dl_fill, dl_color, dl_text) = if is_overdue {
                                (Color32::from_rgb(60, 20, 20),
                                 Color32::from_rgb(239, 68, 68),
                                 format!("⚠ {}", deadline))
                            } else if done {
                                (Color32::from_rgb(30, 50, 30), chart_2, format!("📅 {}", deadline))
                            } else {
                                (Color32::from_rgb(25, 40, 60),
                                 Color32::from_rgb(96, 165, 250),
                                 format!("📅 {}", deadline))
                            };
                            small_badge(ui, &dl_text, dl_fill, dl_color);
                            ui.add_space(4.0);
                        }

                        // Project badge
                        if let Some(proj) = &item.project_name {
                            small_badge(ui,
                                &format!("📁 {}", proj),
                                Color32::from_rgb(40, 40, 60),
                                Color32::from_rgb(148, 130, 240));
                            ui.add_space(4.0);
                        }

                        // Priority badge
                        let (prio_fill, prio_col) = match item.priority {
                            TodoPriority::High   => (Color32::from_rgb(60, 20, 20), Color32::from_rgb(239, 68, 68)),
                            TodoPriority::Medium => (Color32::from_rgb(55, 45, 10), Color32::from_rgb(245, 158, 11)),
                            TodoPriority::Low    => (Color32::from_rgb(25, 50, 25), Color32::from_rgb(132, 204, 22)),
                        };
                        small_badge(ui,
                            &format!("{} {}", item.priority.emoji(), item.priority.label()),
                            prio_fill, prio_col);
                    });
                });

                // Action buttons
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Delete button
                    let id3 = id.clone();
                    if state.todo_state.confirm_delete_id.as_deref() == Some(&id3) {
                        if ui.add(egui::Button::new(RichText::new("Confirm").color(Color32::WHITE).size(11.0))
                            .fill(Color32::from_rgb(180, 40, 40))
                            .min_size(Vec2::new(60.0, 26.0))).clicked() {
                            state.delete_todo_sync(&id3);
                            ctx.request_repaint();
                        }
                        ui.add_space(4.0);
                        if ui.add(egui::Button::new(RichText::new("✕").color(muted).size(11.0))
                            .fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(24.0, 26.0))).clicked() {
                            state.todo_state.confirm_delete_id = None;
                        }
                    } else {
                        if ui.add(egui::Button::new(RichText::new("🗑").color(Color32::from_rgb(239, 68, 68)).size(12.0))
                            .fill(Color32::from_rgb(55, 25, 25))
                            .min_size(Vec2::new(28.0, 26.0))).clicked() {
                            state.todo_state.confirm_delete_id = Some(id3);
                        }
                        ui.add_space(4.0);
                        // Edit button
                        let id4 = id.clone();
                        if !done {
                            if ui.add(egui::Button::new(RichText::new("✏").color(fg).size(12.0))
                                .fill(Color32::from_rgb(55, 55, 68))
                                .min_size(Vec2::new(28.0, 26.0))).clicked() {
                                state.start_edit_todo(&id4);
                            }
                            ui.add_space(4.0);
                        }
                        // Status cycle button
                        let id5 = id.clone();
                        let next_status = match item.status {
                            TodoStatus::Todo       => "→ Start",
                            TodoStatus::InProgress => "→ Done",
                            TodoStatus::Done       => "↺ Reset",
                        };
                        let (ns_fill, ns_fg) = match item.status {
                            TodoStatus::Todo       => (Color32::from_rgb(40, 55, 80), Color32::from_rgb(96, 165, 250)),
                            TodoStatus::InProgress => (Color32::from_rgb(30, 55, 30), chart_2),
                            TodoStatus::Done       => (Color32::from_rgb(55, 55, 68), muted),
                        };
                        if ui.add(egui::Button::new(RichText::new(next_status).color(ns_fg).size(10.0))
                            .fill(ns_fill)
                            .min_size(Vec2::new(60.0, 26.0))).clicked()
                        {
                            let new_status = match item.status {
                                TodoStatus::Todo       => TodoStatus::InProgress,
                                TodoStatus::InProgress => TodoStatus::Done,
                                TodoStatus::Done       => TodoStatus::Todo,
                            };
                            if let Some(i) = state.todo_state.items.iter_mut().find(|i| i.id == id5) {
                                i.status = new_status;
                            }
                            ctx.request_repaint();
                        }
                    }
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  EMPTY STATE
// ─────────────────────────────────────────────────────────────────────────────

fn empty_state(
    ui: &mut egui::Ui,
    filter: &TodoFilter,
    content_width: f32,
    card: Color32, border: Color32, fg: Color32, muted: Color32,
) {
    let (icon, title, subtitle) = match filter {
        TodoFilter::All         => ("✅", "No tasks yet", "Click '＋ New Task' to create your first task"),
        TodoFilter::Active      => ("⬜", "No pending tasks", "All your tasks are in progress or completed"),
        TodoFilter::InProgress  => ("🔄", "Nothing in progress", "Start working on a task to see it here"),
        TodoFilter::Done        => ("🎉", "No completed tasks", "Complete some tasks to see them here"),
        TodoFilter::HighPriority => ("🔴", "No high-priority tasks", "Mark tasks as high priority to see them here"),
        TodoFilter::HasDeadline  => ("📅", "No tasks with deadlines", "Add a deadline to a task to see it here"),
    };

    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(32.0))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(icon).size(40.0));
                ui.add_space(8.0);
                ui.label(RichText::new(title).color(fg).size(16.0).strong());
                ui.add_space(4.0);
                ui.label(RichText::new(subtitle).color(muted).size(12.0));
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  FILTER HELPER
// ─────────────────────────────────────────────────────────────────────────────

fn filtered_items(state: &AppState) -> Vec<TodoItem> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    state.todo_state.items.iter().filter(|item| {
        match &state.todo_state.filter {
            TodoFilter::All         => true,
            TodoFilter::Active      => item.status == TodoStatus::Todo,
            TodoFilter::InProgress  => item.status == TodoStatus::InProgress,
            TodoFilter::Done        => item.status == TodoStatus::Done,
            TodoFilter::HighPriority => item.priority == TodoPriority::High,
            TodoFilter::HasDeadline  => item.deadline.is_some(),
        }
    }).cloned().collect()
}

// ─────────────────────────────────────────────────────────────────────────────
//  BADGE HELPERS
// ─────────────────────────────────────────────────────────────────────────────

fn status_badge(ui: &mut egui::Ui, status: &TodoStatus, chart_2: Color32, chart_3: Color32, muted: Color32) {
    let (fill, color, label) = match status {
        TodoStatus::Todo       => (Color32::from_rgb(45, 45, 58), muted,    "⬜ To Do"),
        TodoStatus::InProgress => (Color32::from_rgb(50, 45, 20), chart_3,  "🔄 In Progress"),
        TodoStatus::Done       => (Color32::from_rgb(25, 50, 25), chart_2,  "✅ Done"),
    };
    small_badge(ui, label, fill, color);
}

fn small_badge(ui: &mut egui::Ui, text: &str, fill: Color32, color: Color32) {
    Frame::none()
        .fill(fill)
        .inner_margin(Margin::symmetric(6.0, 2.0))
        .rounding(Rounding::same(20.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).size(10.5));
        });
}