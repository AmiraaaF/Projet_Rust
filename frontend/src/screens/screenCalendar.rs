use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, ScrollArea, Stroke, Vec2};
use crate::state::{AppState, Screen, TodoStatus};
use crate::screens::screenDashboard::{sidebar_item, sidebar_item_with_badge};
use chrono::{Datelike, Local};

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

pub fn calendar_screen(ctx: &egui::Context, state: &mut AppState) {
    state.poll_notifications_sync();
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
    let chart_1        = state.theme.chart_1;

    // ── TOP BAR ──────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("calendar_top")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📅 Calendar").color(fg).size(18.0).strong());
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
    egui::SidePanel::left("calendar_sidebar")
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
            if sidebar_item(ui, "✅ To-Do",        false, fg, primary) { state.go_to(Screen::Todo); }
            ui.add_space(4.0);
            if sidebar_item(ui, "📅 Calendar",     false, fg, primary) { state.go_to(Screen::Calendar); }
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing",      false, fg, primary) { state.go_to(Screen::Billing); }
            ui.add_space(4.0);
            if sidebar_item_with_badge(ui, "🔔 Notifications", false, fg, primary, state.notif_state.unread_count) {
                state.go_to(Screen::Notifications);
            }
            ui.add_space(4.0);
            if sidebar_item(ui, "👤 Profile",      false, fg, primary) { state.go_to(Screen::Profile); }

            // ── Mini legend in sidebar ─────────────────────────────────────
            ui.add_space(24.0);
            ui.label(RichText::new("LEGEND").color(muted).size(11.0));
            ui.add_space(6.0);
            legend_item(ui, chart_2,                          "Completed");
            ui.add_space(3.0);
            legend_item(ui, Color32::from_rgb(239, 68, 68),  "Overdue");
            ui.add_space(3.0);
            legend_item(ui, chart_3,                          "In Progress");
            ui.add_space(3.0);
            legend_item(ui, Color32::from_rgb(96, 165, 250), "To Do");
            ui.add_space(3.0);
            legend_item(ui, chart_1,                          "Today");
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

                    // ── Page header ───────────────────────────────────────
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Task Calendar").color(fg).size(22.0).strong());
                            ui.add_space(2.0);
                            ui.label(RichText::new("View your task deadlines and plan ahead")
                                .color(muted).size(13.0));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let today = Local::now();
                            let is_current_month = state.calendar_state.year == today.year()
                                && state.calendar_state.month == today.month();
                            if !is_current_month {
                                if ui.add(egui::Button::new(
                                    RichText::new("Today").color(primary_fg).size(12.0)
                                ).fill(primary).min_size(Vec2::new(60.0, 28.0))).clicked() {
                                    state.calendar_state.year  = today.year();
                                    state.calendar_state.month = today.month();
                                    state.calendar_state.selected_day = None;
                                }
                            }
                        });
                    });
                    ui.add_space(20.0);

                    // ── Month navigation ──────────────────────────────────
                    month_nav(ui, state, fg, muted, border, card, primary, primary_fg);
                    ui.add_space(16.0);

                    // ── Calendar grid ─────────────────────────────────────
                    let sel_day = state.calendar_state.selected_day;
                    calendar_grid(ui, ctx, state, content_width, fg, muted, border, card,
                                  primary, primary_fg, chart_2, chart_3, chart_1);
                    ui.add_space(16.0);

                    // ── Selected day tasks panel ───────────────────────────
                    if let Some(day) = state.calendar_state.selected_day {
                        day_detail_panel(ui, state, day, content_width,
                                         fg, muted, border, card, primary, primary_fg,
                                         chart_2, chart_3);
                        ui.add_space(16.0);
                    }

                    // ── Upcoming deadlines ────────────────────────────────
                    upcoming_panel(ui, state, content_width, fg, muted, border, card,
                                   primary, primary_fg, chart_2, chart_3);
                    ui.add_space(24.0);
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  MONTH NAVIGATION
// ─────────────────────────────────────────────────────────────────────────────

fn month_nav(
    ui: &mut egui::Ui, state: &mut AppState,
    fg: Color32, muted: Color32, border: Color32, card: Color32,
    primary: Color32, primary_fg: Color32,
) {
    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::symmetric(16.0, 12.0))
        .rounding(Rounding::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(
                    RichText::new("◀").color(fg).size(16.0))
                    .fill(Color32::from_rgb(55, 55, 68))
                    .min_size(Vec2::new(32.0, 32.0))).clicked()
                {
                    state.calendar_state.prev_month();
                }
                ui.add_space(16.0);
                ui.label(RichText::new(format!(
                    "{} {}",
                    state.calendar_state.month_name(),
                    state.calendar_state.year
                )).color(fg).size(20.0).strong());
                ui.add_space(16.0);
                if ui.add(egui::Button::new(
                    RichText::new("▶").color(fg).size(16.0))
                    .fill(Color32::from_rgb(55, 55, 68))
                    .min_size(Vec2::new(32.0, 32.0))).clicked()
                {
                    state.calendar_state.next_month();
                }

                // Summary stats for the month
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (year, month) = (state.calendar_state.year, state.calendar_state.month);
                    let month_str = format!("{:04}-{:02}", year, month);
                    let month_tasks: Vec<_> = state.todo_state.items.iter()
                        .filter(|i| i.deadline.as_ref().map(|d| d.starts_with(&month_str)).unwrap_or(false))
                        .collect();
                    let done_ct   = month_tasks.iter().filter(|i| i.status == TodoStatus::Done).count();
                    let total_ct  = month_tasks.len();

                    ui.label(RichText::new(format!("{}/{} done", done_ct, total_ct))
                        .color(muted).size(12.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!("{} tasks this month", total_ct))
                        .color(muted).size(12.0));
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  CALENDAR GRID
// ─────────────────────────────────────────────────────────────────────────────

fn calendar_grid(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    state: &mut AppState,
    content_width: f32,
    fg: Color32, muted: Color32, border: Color32, card: Color32,
    primary: Color32, primary_fg: Color32,
    chart_2: Color32, chart_3: Color32, chart_1: Color32,
) {
    let today     = Local::now();
    let today_day = if today.year() == state.calendar_state.year
        && today.month() == state.calendar_state.month
    { Some(today.day()) } else { None };

    let first_wd   = state.calendar_state.first_weekday(); // 0=Mon
    let days_count = state.calendar_state.days_in_month();
    let year       = state.calendar_state.year;
    let month      = state.calendar_state.month;

    // ── Weekday headers ───────────────────────────────────────────────────────
    let cell_w = (content_width - 12.0) / 7.0;
    let cell_h = 88.0_f32;

    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);

            // Header row
            ui.horizontal(|ui| {
                ui.set_min_height(30.0);
                for &day_name in &WEEKDAYS {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(cell_w, 30.0), egui::Sense::hover());
                    let is_weekend = day_name == "Sat" || day_name == "Sun";
                    ui.painter().rect_filled(
                        rect, Rounding::ZERO,
                        Color32::from_rgb(45, 45, 58),
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        day_name,
                        egui::FontId::proportional(12.0),
                        if is_weekend { muted } else { fg },
                    );
                }
            });

            // Divider
            ui.painter().hline(
                ui.available_rect_before_wrap().x_range(),
                ui.cursor().top(),
                Stroke::new(1.0, border),
            );

            // Day cells
            let mut current_day = 1u32;
            let total_cells = first_wd + days_count;
            let rows = ((total_cells + 6) / 7) as usize;

            for row in 0..rows {
                ui.horizontal(|ui| {
                    for col in 0..7u32 {
                        let cell_idx = (row as u32) * 7 + col;
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(cell_w, cell_h),
                            egui::Sense::click(),
                        );

                        let is_valid = cell_idx >= first_wd && current_day <= days_count;
                        let day_num  = if is_valid { current_day } else { 0 };
                        let is_today = today_day == Some(day_num) && is_valid;
                        let is_sel   = state.calendar_state.selected_day == Some(day_num) && is_valid;

                        // Background
                        let bg_fill = if is_sel      { Color32::from_rgb(60, 50, 90) }
                                      else if is_today { Color32::from_rgb(50, 45, 75) }
                                      else             { card };
                        ui.painter().rect_filled(rect, Rounding::ZERO, bg_fill);

                        // Border right/bottom
                        if col < 6 {
                            ui.painter().vline(rect.right(), rect.y_range(), Stroke::new(1.0, border));
                        }
                        ui.painter().hline(rect.x_range(), rect.bottom(), Stroke::new(1.0, border));

                        if is_valid {
                            // Day number
                            let num_color = if is_today   { chart_1 }
                                            else if is_sel { Color32::WHITE }
                                            else           { fg };
                            if is_today {
                                let circ_center = egui::pos2(rect.left() + 14.0, rect.top() + 14.0);
                                ui.painter().circle_filled(circ_center, 11.0, chart_1);
                                ui.painter().text(
                                    circ_center,
                                    egui::Align2::CENTER_CENTER,
                                    format!("{}", day_num),
                                    egui::FontId::proportional(11.0),
                                    Color32::BLACK,
                                );
                            } else {
                                ui.painter().text(
                                    egui::pos2(rect.left() + 14.0, rect.top() + 14.0),
                                    egui::Align2::CENTER_CENTER,
                                    format!("{}", day_num),
                                    egui::FontId::proportional(11.0),
                                    num_color,
                                );
                            }

                            // Task dots for this day
                            let day_str = format!("{:04}-{:02}-{:02}", year, month, day_num);
                            let day_tasks: Vec<_> = state.todo_state.items.iter()
                                .filter(|t| t.deadline.as_deref() == Some(&day_str))
                                .collect();

                            if !day_tasks.is_empty() {
                                let max_show = 3;
                                let show_count = day_tasks.len().min(max_show);
                                let dot_y = rect.top() + 30.0;
                                let dot_gap = 4.0;
                                let dot_r = 3.5_f32;
                                let start_x = rect.left() + 6.0;

                                for (di, task) in day_tasks.iter().take(show_count).enumerate() {
                                    let dx = start_x + di as f32 * (dot_r * 2.0 + dot_gap);
                                    let today_str = today.format("%Y-%m-%d").to_string();
                                    let dot_color = if task.status == TodoStatus::Done { chart_2 }
                                        else if day_str.as_str() < today_str.as_str() { Color32::from_rgb(239, 68, 68) }
                                        else if task.status == TodoStatus::InProgress  { chart_3 }
                                        else { Color32::from_rgb(96, 165, 250) };
                                    ui.painter().circle_filled(egui::pos2(dx, dot_y), dot_r, dot_color);
                                }

                                if day_tasks.len() > max_show {
                                    ui.painter().text(
                                        egui::pos2(start_x + max_show as f32 * (dot_r * 2.0 + dot_gap), dot_y),
                                        egui::Align2::LEFT_CENTER,
                                        format!("+{}", day_tasks.len() - max_show),
                                        egui::FontId::proportional(8.0),
                                        muted,
                                    );
                                }

                                // First task title (truncated)
                                if let Some(first) = day_tasks.first() {
                                    let t = if first.title.len() > 14 {
                                        format!("{}…", &first.title[..12])
                                    } else {
                                        first.title.clone()
                                    };
                                    let today_str = today.format("%Y-%m-%d").to_string();
                                    let t_color = if first.status == TodoStatus::Done { muted }
                                        else if day_str.as_str() < today_str.as_str() { Color32::from_rgb(239, 68, 68) }
                                        else { fg };
                                    ui.painter().text(
                                        egui::pos2(rect.left() + 5.0, dot_y + 12.0),
                                        egui::Align2::LEFT_CENTER,
                                        t,
                                        egui::FontId::proportional(9.0),
                                        t_color,
                                    );
                                }

                                if day_tasks.len() > 1 {
                                    ui.painter().text(
                                        egui::pos2(rect.left() + 5.0, dot_y + 24.0),
                                        egui::Align2::LEFT_CENTER,
                                        format!("+ {} more", day_tasks.len() - 1),
                                        egui::FontId::proportional(8.5),
                                        muted,
                                    );
                                }
                            }

                            // Click handler
                            if response.clicked() {
                                if state.calendar_state.selected_day == Some(day_num) {
                                    state.calendar_state.selected_day = None;
                                } else {
                                    state.calendar_state.selected_day = Some(day_num);
                                }
                                ctx.request_repaint();
                            }

                            // Hover highlight
                            if response.hovered() && !is_sel {
                                ui.painter().rect_filled(
                                    rect,
                                    Rounding::ZERO,
                                    Color32::from_rgba_premultiplied(255, 255, 255, 8),
                                );
                            }

                            if is_valid { current_day += 1; }
                        } else {
                            // Empty cell
                            ui.painter().rect_filled(rect, Rounding::ZERO, Color32::from_rgb(30, 30, 35));
                        }
                    }
                });

                if current_day > days_count { break; }
            }
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  DAY DETAIL PANEL
// ─────────────────────────────────────────────────────────────────────────────

fn day_detail_panel(
    ui: &mut egui::Ui,
    state: &AppState,
    day: u32,
    content_width: f32,
    fg: Color32, muted: Color32, border: Color32, card: Color32,
    primary: Color32, primary_fg: Color32,
    chart_2: Color32, chart_3: Color32,
) {
    let year      = state.calendar_state.year;
    let month     = state.calendar_state.month;
    let day_str   = format!("{:04}-{:02}-{:02}", year, month, day);
    let today_str = Local::now().format("%Y-%m-%d").to_string();

    let tasks: Vec<_> = state.todo_state.items.iter()
        .filter(|t| t.deadline.as_deref() == Some(&day_str))
        .collect();

    let month_name = match month {
        1=>"Jan",2=>"Feb",3=>"Mar",4=>"Apr",5=>"May",6=>"Jun",
        7=>"Jul",8=>"Aug",9=>"Sep",10=>"Oct",11=>"Nov",_=>"Dec",
    };

    Frame::none()
        .fill(Color32::from_rgb(42, 42, 55))
        .stroke(Stroke::new(1.5, primary))
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);

            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("📅 {} {}, {}", month_name, day, year))
                    .color(fg).size(16.0).strong());
                ui.add_space(8.0);
                if day_str == today_str {
                    Frame::none()
                        .fill(Color32::from_rgb(124, 58, 202))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .rounding(Rounding::same(20.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("Today").color(Color32::WHITE).size(10.0));
                        });
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} task{}", tasks.len(), if tasks.len() == 1 { "" } else { "s" }))
                        .color(muted).size(12.0));
                });
            });

            ui.add_space(12.0);

            if tasks.is_empty() {
                ui.label(RichText::new("No tasks due on this day")
                    .color(muted).size(13.0));
                ui.add_space(4.0);
                ui.label(RichText::new("Go to the To-Do screen to add a task with this deadline")
                    .color(muted).size(11.0));
            } else {
                for task in &tasks {
                    Frame::none()
                        .fill(card)
                        .stroke(Stroke::new(1.0, border))
                        .inner_margin(Margin::symmetric(12.0, 8.0))
                        .rounding(Rounding::same(6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Status icon
                                ui.label(RichText::new(task.status.icon()).size(14.0));
                                ui.add_space(6.0);
                                // Priority dot
                                let pc = match task.priority {
                                    crate::state::TodoPriority::High   => Color32::from_rgb(239, 68, 68),
                                    crate::state::TodoPriority::Medium => Color32::from_rgb(245, 158, 11),
                                    crate::state::TodoPriority::Low    => Color32::from_rgb(132, 204, 22),
                                };
                                let (pr, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                                ui.painter().circle_filled(pr.center(), 4.0, pc);
                                ui.add_space(6.0);
                                // Title
                                let done = task.status == TodoStatus::Done;
                                ui.label(if done {
                                    RichText::new(&task.title).color(muted).size(13.0).strikethrough()
                                } else {
                                    RichText::new(&task.title).color(fg).size(13.0)
                                });
                                // Project
                                if let Some(proj) = &task.project_name {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(format!("📁 {}", proj))
                                            .color(muted).size(11.0));
                                    });
                                }
                            });
                            if !task.description.is_empty() {
                                ui.add_space(2.0);
                                ui.label(RichText::new(&task.description).color(muted).size(11.0));
                            }
                        });
                    ui.add_space(4.0);
                }
            }
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  UPCOMING DEADLINES PANEL
// ─────────────────────────────────────────────────────────────────────────────

fn upcoming_panel(
    ui: &mut egui::Ui,
    state: &AppState,
    content_width: f32,
    fg: Color32, muted: Color32, border: Color32, card: Color32,
    primary: Color32, primary_fg: Color32,
    chart_2: Color32, chart_3: Color32,
) {
    let today_str = Local::now().format("%Y-%m-%d").to_string();

    // Gather upcoming tasks (next 14 days, not done)
    let mut upcoming: Vec<_> = state.todo_state.items.iter()
        .filter(|t| {
            t.status != TodoStatus::Done &&
            t.deadline.as_ref().map(|d| d.as_str() >= today_str.as_str()).unwrap_or(false)
        })
        .collect();
    upcoming.sort_by(|a, b| a.deadline.cmp(&b.deadline));
    let upcoming = &upcoming[..upcoming.len().min(8)];

    let mut overdue: Vec<_> = state.todo_state.items.iter()
        .filter(|t| {
            t.status != TodoStatus::Done &&
            t.deadline.as_ref().map(|d| d.as_str() < today_str.as_str()).unwrap_or(false)
        })
        .collect();
    overdue.sort_by(|a, b| a.deadline.cmp(&b.deadline));

    ui.label(RichText::new("Schedule Overview").color(fg).size(16.0).strong());
    ui.add_space(4.0);
    ui.label(RichText::new("Upcoming and overdue tasks at a glance").color(muted).size(13.0));
    ui.add_space(12.0);

    let panel_w = if content_width > 700.0 { (content_width - 16.0) / 2.0 } else { content_width };

    ui.horizontal_wrapped(|ui| {
        ui.set_max_width(content_width);

        // Upcoming panel
        Frame::none()
            .fill(card)
            .stroke(Stroke::new(1.0, border))
            .inner_margin(Margin::same(16.0))
            .rounding(Rounding::same(12.0))
            .show(ui, |ui| {
                ui.set_min_width(panel_w - 8.0);
                ui.set_max_width(panel_w - 8.0);
                ui.label(RichText::new("📅 Upcoming").color(fg).size(14.0).strong());
                ui.add_space(8.0);
                if upcoming.is_empty() {
                    ui.label(RichText::new("No upcoming tasks with deadlines").color(muted).size(12.0));
                } else {
                    for task in upcoming {
                        upcoming_task_row(ui, task, false, fg, muted, border, chart_2, chart_3);
                        ui.add_space(4.0);
                    }
                }
            });

        ui.add_space(16.0);

        // Overdue panel
        Frame::none()
            .fill(if overdue.is_empty() { card } else { Color32::from_rgb(42, 28, 28) })
            .stroke(Stroke::new(1.0, if overdue.is_empty() { border } else { Color32::from_rgb(239, 68, 68) }))
            .inner_margin(Margin::same(16.0))
            .rounding(Rounding::same(12.0))
            .show(ui, |ui| {
                ui.set_min_width(panel_w - 8.0);
                ui.set_max_width(panel_w - 8.0);
                ui.label(RichText::new(
                    if overdue.is_empty() { "✅ No Overdue Tasks" } else { "⚠ Overdue Tasks" }
                ).color(if overdue.is_empty() { chart_2 } else { Color32::from_rgb(239, 68, 68) })
                 .size(14.0).strong());
                ui.add_space(8.0);
                if overdue.is_empty() {
                    ui.label(RichText::new("Great job — you're on track!").color(muted).size(12.0));
                } else {
                    for task in &overdue {
                        upcoming_task_row(ui, task, true, fg, muted, border, chart_2, chart_3);
                        ui.add_space(4.0);
                    }
                }
            });
    });
}

fn upcoming_task_row(
    ui: &mut egui::Ui, task: &&crate::state::TodoItem,
    is_overdue: bool,
    fg: Color32, muted: Color32, border: Color32,
    chart_2: Color32, chart_3: Color32,
) {
    let bg = if is_overdue {
        Color32::from_rgb(55, 28, 28)
    } else {
        Color32::from_rgb(42, 42, 52)
    };
    Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::symmetric(10.0, 6.0))
        .rounding(Rounding::same(6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let pc = match task.priority {
                    crate::state::TodoPriority::High   => Color32::from_rgb(239, 68, 68),
                    crate::state::TodoPriority::Medium => Color32::from_rgb(245, 158, 11),
                    crate::state::TodoPriority::Low    => Color32::from_rgb(132, 204, 22),
                };
                let (pr, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                ui.painter().circle_filled(pr.center(), 4.0, pc);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&task.title).color(fg).size(12.5));
                    if let Some(dl) = &task.deadline {
                        let color = if is_overdue { Color32::from_rgb(239, 68, 68) }
                                    else { Color32::from_rgb(96, 165, 250) };
                        ui.label(RichText::new(format!("📅 {}", dl)).color(color).size(10.5));
                    }
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  HELPERS
// ─────────────────────────────────────────────────────────────────────────────

fn legend_item(ui: &mut egui::Ui, color: Color32, label: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), egui::Sense::hover());
        ui.painter().circle_filled(r.center(), 5.0, color);
        ui.add_space(6.0);
        ui.label(RichText::new(label).color(Color32::from_rgb(180, 180, 180)).size(11.0));
    });
}