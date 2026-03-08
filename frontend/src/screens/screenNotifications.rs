use eframe::egui::{
    self, Color32, Frame, Margin, RichText, Rounding, ScrollArea, Stroke, Vec2,
};
use crate::state::{AppState, Screen};
use crate::screens::screenDashboard::{sidebar_item, sidebar_item_with_badge};

// ─────────────────────────────────────────────────────────────────────────────
//  TYPES
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NotifType {
    InApp,
    Email,
    System,
}

impl NotifType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "email"  => NotifType::Email,
            "system" => NotifType::System,
            _        => NotifType::InApp,
        }
    }
    pub fn icon(&self) -> &str {
        match self {
            NotifType::InApp  => "🔔",
            NotifType::Email  => "📧",
            NotifType::System => "⚙",
        }
    }
    pub fn label(&self) -> &str {
        match self {
            NotifType::InApp  => "In-App",
            NotifType::Email  => "Email",
            NotifType::System => "System",
        }
    }
    pub fn color(&self) -> Color32 {
        match self {
            NotifType::InApp  => Color32::from_rgb(124, 58, 202),
            NotifType::Email  => Color32::from_rgb(59, 130, 246),
            NotifType::System => Color32::from_rgb(245, 158, 11),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotifStatus {
    Sent,
    Read,
    Failed,
}

impl NotifStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "read"   => NotifStatus::Read,
            "failed" => NotifStatus::Failed,
            _        => NotifStatus::Sent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id:    String,
    pub title: String,
    pub message: String,
    pub notif_type: NotifType,
    pub status: NotifStatus,
    pub created_at: String,
}

// ─────────────────────────────────────────────────────────────────────────────
//  FILTER TAB
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FilterTab {
    All,
    Unread,
    InApp,
    System,
}

impl FilterTab {
    pub fn label(&self) -> &str {
        match self {
            FilterTab::All    => "All",
            FilterTab::Unread => "Unread",
            FilterTab::InApp  => "Projects & Tasks",
            FilterTab::System => "System",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  MAIN SCREEN
// ─────────────────────────────────────────────────────────────────────────────

pub fn notifications_screen(ctx: &egui::Context, state: &mut AppState) {
    state.load_notifications_sync();
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

    // ── TOP BAR ──────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("notif_top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔔 Notifications").color(fg).size(18.0).strong());

                // Unread badge
                let unread = state.notif_state.unread_count;
                if unread > 0 {
                    ui.add_space(6.0);
                    Frame::none()
                        .fill(Color32::from_rgb(239, 68, 68))
                        .inner_margin(Margin::symmetric(6.0, 2.0))
                        .rounding(Rounding::same(20.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("{}", unread))
                                .color(Color32::WHITE).size(11.0).strong());
                        });
                }

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
    egui::SidePanel::left("notif_sidebar")
        .show_separator_line(false)
        .min_width(180.0).max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);
            if sidebar_item(ui, "📊 Dashboard",     false, fg, primary) { state.go_to(Screen::Dashboard); }
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projects",      false, fg, primary) { state.go_to(Screen::Projects); }
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing",       false, fg, primary) { state.go_to(Screen::Billing); }
            ui.add_space(4.0);
            sidebar_item_with_badge(ui, "🔔 Notifications", true, fg, primary, state.notif_state.unread_count);
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
                        ui.set_max_width(content_width);
                        ui.vertical(|ui| {
                            ui.set_max_width(content_width * 0.6); 
                            ui.label(RichText::new("Notifications").color(fg).size(22.0).strong());
                            ui.add_space(2.0);
                            ui.label(RichText::new("Stay on top of everything happening in your workspace")
                                .color(muted).size(13.0));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.set_max_width(content_width * 0.35); 
                            if state.notif_state.unread_count > 0 {
                                if ui.add(egui::Button::new(
                                    RichText::new("✅ Mark all read").color(primary_fg).size(12.0))
                                    .fill(primary)
                                    .min_size(Vec2::new(120.0, 30.0))).clicked()
                                {
                                    state.mark_all_read_sync();
                                    ctx.request_repaint();
                                }
                            }
                        });
                    });
                    ui.add_space(20.0);

                    // ── Stats row ─────────────────────────────────────────
                    stats_row(ui, state, content_width, card, fg, muted, border, chart_2, chart_3);
                    ui.add_space(20.0);

                    // ── Filter tabs ───────────────────────────────────────
                    filter_tabs(ui, state, content_width, fg, primary, muted, border);
                    ui.add_space(16.0);

                    // ── Notification list ─────────────────────────────────
                    let notifications = filtered_notifications(state);
                    if notifications.is_empty() {
                        empty_state(ui, &state.notif_state.active_filter, content_width, card, border, fg, muted);
                    } else {
                        let n = notifications.len();
                        for (i, notif) in notifications.iter().enumerate() {
                            notification_row(ui, ctx, notif, state, content_width, card, fg, muted, border, primary, primary_fg, chart_2);
                            if i < n - 1 { ui.add_space(4.0); }
                        }
                    }
                    ui.add_space(24.0);

                    // ── Clear read button ─────────────────────────────────
                    let has_read = state.notif_state.notifications.iter().any(|n| n.status == NotifStatus::Read);
                    if has_read {
                        if ui.add(egui::Button::new(
                            RichText::new("🗑 Clear read notifications").color(Color32::from_rgb(239, 68, 68)).size(12.0))
                            .fill(Color32::from_rgb(60, 30, 30))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                            .min_size(Vec2::new(200.0, 30.0))).clicked()
                        {
                            state.clear_read_notifications_sync();
                            ctx.request_repaint();
                        }
                    }

                    // ── Toast ─────────────────────────────────────────────
                    if let Some(msg) = state.notif_state.toast_message.clone() {
                        if state.notif_state.toast_time.elapsed().as_secs_f32() > 5.0 {
                            state.notif_state.toast_message = None;
                        } else {
                            ui.add_space(12.0);
                            ui.vertical(|ui| {
                                ui.set_max_width(content_width);
                                let (fill, stroke, text_color) = if msg.starts_with("✅") {
                                    (Color32::from_rgb(30, 55, 30), chart_2, chart_2)
                                } else {
                                    (Color32::from_rgb(55, 22, 22), Color32::from_rgb(239, 68, 68), Color32::from_rgb(239, 68, 68))
                                };
                                Frame::none()
                                    .fill(fill)
                                    .stroke(Stroke::new(1.0, stroke))
                                    .inner_margin(Margin::symmetric(16.0, 10.0))
                                    .rounding(Rounding::same(8.0))
                                    .show(ui, |ui| {
                                        ui.set_max_width(content_width);
                                        ui.horizontal(|ui| {
                                            ui.set_max_width(content_width);
                                            ui.label(RichText::new(&msg).color(text_color).size(13.0));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("X").clicked() {
                                                    state.notif_state.toast_message = None;
                                                }
                                            });
                                        });
                                    });
                            });
                            ctx.request_repaint();
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
    ui: &mut egui::Ui,
    state: &AppState,
    content_w: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    _chart_2: Color32, _chart_3: Color32,
) {
    let total  = state.notif_state.notifications.len();
    let unread = state.notif_state.unread_count as usize;
    let inapp  = state.notif_state.notifications.iter().filter(|n| n.notif_type == NotifType::InApp).count();
    let system = state.notif_state.notifications.iter().filter(|n| n.notif_type == NotifType::System).count();

    ui.vertical(|ui| {
        ui.set_max_width(content_w);
        let num_cols = if content_w > 900.0 {
            4
        } else if content_w > 550.0 {
            2
        } else {
            1
        };
        
        ui.columns(num_cols, |columns| {
            let stats = vec![
                ("📬 Total", total.to_string()),
                ("🔴 Unread", unread.to_string()),
                ("📁 Projects & Tasks", inapp.to_string()),
                ("⚙ System", system.to_string()),
            ];
            
            for (idx, (label, value)) in stats.iter().enumerate() {
                let col_idx = idx % num_cols;
                stat_mini_card(&mut columns[col_idx], label, &value, card, fg, muted, border);
            }
        });
    });
}

fn stat_mini_card(
    ui: &mut egui::Ui, label: &str, value: &str,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
) {
    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(8.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.label(RichText::new(label).color(muted).size(12.0));
            ui.add_space(2.0);
            ui.label(RichText::new(value).color(fg).size(24.0).strong());
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  FILTER TABS
// ─────────────────────────────────────────────────────────────────────────────

fn filter_tabs(
    ui: &mut egui::Ui,
    state: &mut AppState,
    content_width: f32,
    fg: Color32, primary: Color32, muted: Color32, border: Color32,
) {
    ui.columns(1, |columns| {
        columns[0].horizontal_wrapped(|ui| {
            for tab in &[FilterTab::All, FilterTab::Unread, FilterTab::InApp, FilterTab::System] {
                let is_active = &state.notif_state.active_filter == tab;
                let (fill, text_color, stroke_color) = if is_active {
                    (Color32::from_rgb(124, 58, 202), Color32::WHITE, Color32::from_rgb(124, 58, 202))
                } else {
                    (Color32::TRANSPARENT, muted, border)
                };

                let btn = egui::Button::new(RichText::new(tab.label()).color(text_color).size(13.0))
                    .fill(fill)
                    .stroke(Stroke::new(1.0, stroke_color))
                    .min_size(Vec2::new(76.0, 30.0));

                if ui.add(btn).clicked() {
                    state.notif_state.active_filter = tab.clone();
                }
                ui.add_space(4.0);
            }
        });
    });
}

// ─────────────────────────────────────────────────────────────────────────────
//  FILTER HELPER
// ─────────────────────────────────────────────────────────────────────────────

fn filtered_notifications(state: &AppState) -> Vec<Notification> {
    state.notif_state.notifications.iter().filter(|n| {
        match &state.notif_state.active_filter {
            FilterTab::All    => true,
            FilterTab::Unread => n.status == NotifStatus::Sent,
            FilterTab::InApp  => n.notif_type == NotifType::InApp,
            FilterTab::System => n.notif_type == NotifType::System,
        }
    }).cloned().collect()
}

// ─────────────────────────────────────────────────────────────────────────────
//  NOTIFICATION ROW
// ─────────────────────────────────────────────────────────────────────────────

fn notification_row(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    notif: &Notification,
    state: &mut AppState,
    content_width: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    primary: Color32, primary_fg: Color32,
    chart_2: Color32,
) {
    let is_unread  = notif.status == NotifStatus::Sent;
    let row_fill   = if is_unread { Color32::from_rgb(45, 42, 58) } else { card };
    let row_border = if is_unread { Color32::from_rgb(124, 58, 202) } else { border };

    Frame::none()
        .fill(row_fill)
        .stroke(Stroke::new(if is_unread { 1.5 } else { 1.0 }, row_border))
        .inner_margin(Margin::same(10.0))
        .rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);

            ui.horizontal(|ui| {
                // Unread dot
                if is_unread {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.0, Color32::from_rgb(124, 58, 202));
                } else {
                    ui.add_space(8.0);
                }
                ui.add_space(6.0);

                
                Frame::none()
                    .fill(Color32::from_rgba_premultiplied(
                        notif.notif_type.color().r(),
                        notif.notif_type.color().g(),
                        notif.notif_type.color().b(),
                        40,
                    ))
                    .inner_margin(Margin::symmetric(6.0, 4.0))
                    .rounding(Rounding::same(6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(notif.notif_type.icon())
                            .color(notif.notif_type.color()).size(14.0));
                    });
                ui.add_space(10.0);

                // Title + message 
                let text_width = (content_width - 28.0 - 14.0 - 160.0).max(100.0);
                ui.vertical(|ui| {
                    ui.set_max_width(text_width);
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&notif.title)
                            .color(if is_unread { fg } else { muted })
                            .size(13.5).strong());
                        ui.add_space(6.0);
                        Frame::none()
                            .fill(Color32::from_rgba_premultiplied(
                                notif.notif_type.color().r(),
                                notif.notif_type.color().g(),
                                notif.notif_type.color().b(),
                                30,
                            ))
                            .stroke(Stroke::new(1.0, notif.notif_type.color()))
                            .inner_margin(Margin::symmetric(5.0, 1.0))
                            .rounding(Rounding::same(20.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(notif.notif_type.label())
                                    .color(notif.notif_type.color()).size(10.0));
                            });
                    });
                    ui.add_space(3.0);
                    ui.label(RichText::new(&notif.message).color(muted).size(12.0));
                    ui.add_space(3.0);
                    ui.label(RichText::new(&notif.created_at).color(Color32::from_rgb(120, 120, 120)).size(11.0));
                });

                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let notif_id = notif.id.clone();
                    if ui.add(egui::Button::new(RichText::new("🗑").size(12.0).color(Color32::from_rgb(239, 68, 68)))
                        .fill(Color32::from_rgb(60, 30, 30))
                        .min_size(Vec2::new(28.0, 26.0))).clicked()
                    {
                        state.delete_notification_sync(&notif_id);
                        ctx.request_repaint();
                    }
                    ui.add_space(4.0);
                    if is_unread {
                        let notif_id2 = notif.id.clone();
                        if ui.add(egui::Button::new(RichText::new("✅ Read").color(primary_fg).size(11.0))
                            .fill(primary)
                            .min_size(Vec2::new(60.0, 26.0))).clicked()
                        {
                            state.mark_notification_read_sync(&notif_id2);
                            ctx.request_repaint();
                        }
                    } else {
                        Frame::none()
                            .fill(Color32::from_rgb(30, 55, 30))
                            .inner_margin(Margin::symmetric(6.0, 2.0))
                            .rounding(Rounding::same(20.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new("✅ Read").color(chart_2).size(10.0));
                            });
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
    filter: &FilterTab,
    content_width: f32,
    card: Color32, border: Color32, fg: Color32, muted: Color32,
) {
    let (icon, title, subtitle) = match filter {
        FilterTab::All    => ("🔔", "No notifications yet",          "You'll see alerts from billing, projects & tasks here"),
        FilterTab::Unread => ("✅", "All caught up!",               "You have no unread notifications"),
        FilterTab::InApp  => ("📋", "No project notifications",    "You'll see updates from your projects and tasks here"),
        FilterTab::System => ("⚙",  "No system notifications",      "System events will appear here"),
    };

    ui.columns(1, |columns| {
        Frame::none()
            .fill(card)
            .stroke(Stroke::new(1.0, border))
            .inner_margin(Margin::same(24.0))
            .rounding(Rounding::same(8.0))
            .show(&mut columns[0], |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(icon).size(32.0));
                    ui.add_space(8.0);
                    ui.label(RichText::new(title).color(fg).size(14.0).strong());
                    ui.add_space(2.0);
                    ui.label(RichText::new(subtitle).color(muted).size(11.0));
                });
            });
    });
}


pub fn notification_bell(
    ui: &mut egui::Ui,
    unread: u64,
    fg: Color32,
    _primary: Color32,
) -> bool {
    let btn_resp = ui.add(
        egui::Button::new(RichText::new("🔔").size(16.0).color(fg))
            .fill(Color32::TRANSPARENT)
            .min_size(egui::vec2(28.0, 28.0)),
    );

    if unread > 0 {
        let rect = btn_resp.rect;
        let badge_center = egui::pos2(rect.right() - 2.0, rect.top() + 4.0);
        let radius = 8.0;
        ui.painter().circle_filled(badge_center, radius, Color32::from_rgb(239, 68, 68));
        ui.painter().text(
            badge_center,
            egui::Align2::CENTER_CENTER,
            format!("{}", unread.min(99)),
            egui::FontId::proportional(9.0),
            Color32::WHITE,
        );
    }
    btn_resp.clicked()
}