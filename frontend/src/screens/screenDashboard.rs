use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke};
use crate::state::{AppState, Screen};

pub fn dashboard_screen(ctx: &egui::Context, state: &mut AppState) {
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

    if state.current_user.is_some() && !state.notif_state.loaded {
        state.load_notifications_sync();
    }

    state.poll_notifications_sync();
    ctx.request_repaint();

    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📊 Dashboard").color(fg).size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(
                        RichText::new("🔓 Logout").color(destructive_fg).size(13.0)
                    ).fill(destructive)).clicked() { state.logout(); }

                    if let Some(user) = &state.current_user.clone() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(format!("👤 {}", user.name)).color(muted).size(13.0));
                    }
                    ui.add_space(12.0);
                });
            });
        });

    egui::SidePanel::left("sidebar")
        .show_separator_line(false)
        .min_width(180.0).max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);
            sidebar_item(ui, "📊 Dashboard", true, fg, primary);
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projects", false, fg, primary) { state.go_to(Screen::Projects); }
            ui.add_space(4.0);
            if sidebar_item(ui, "💳 Billing", false, fg, primary) { state.go_to(Screen::Billing); }
            ui.add_space(4.0);
            if sidebar_item_with_badge(ui, "🔔 Notifications", false, fg, primary, state.notif_state.unread_count) {
                state.go_to(Screen::Notifications);
            }
        });

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg).inner_margin(Margin::same(24.0)))
        .show(ctx, |ui| {
            ui.label(RichText::new("Overview").color(fg).size(20.0).strong());
            ui.add_space(4.0);
            ui.label(RichText::new("Welcome to your dashboard").color(muted).size(13.0));
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                stat_card(ui, "📁 Projects", &state.projects.len().to_string(), card, fg, muted, border);
                ui.add_space(12.0);
                stat_card(ui, "✅ Tasks", &state.current_tasks.len().to_string(), card, fg, muted, border);
                ui.add_space(12.0);
                stat_card(ui, "💳 Plan", state.billing_state.current_plan.name(), card, fg, muted, border);
                ui.add_space(12.0);
                stat_card(ui, "🔔 Unread", &state.notif_state.unread_count.to_string(), card, fg, muted, border);
            });

            ui.add_space(24.0);
            ui.label(RichText::new("Recent Projects").color(fg).size(16.0).strong());
            ui.add_space(12.0);

            if state.projects.is_empty() {
                Frame::none()
                    .fill(card).stroke(Stroke::new(1.0, border))
                    .inner_margin(Margin::same(20.0)).rounding(Rounding::same(8.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No projects yet").color(muted).size(13.0));
                            ui.add_space(8.0);
                            if ui.add(egui::Button::new(
                                RichText::new("+ Create a project").color(primary_fg)
                            ).fill(primary)).clicked() { state.go_to(Screen::Projects); }
                        });
                    });
            } else {
                let projects_clone = state.projects.clone();
                for project in projects_clone.iter().take(5) {
                    Frame::none()
                        .fill(card).stroke(Stroke::new(1.0, border))
                        .inner_margin(Margin::symmetric(16.0, 10.0)).rounding(Rounding::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("📁 {}", project.name)).color(fg));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(
                                        RichText::new("View").color(primary_fg).size(12.0)
                                    ).fill(primary)).clicked() {
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

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, card: egui::Color32, fg: egui::Color32, muted: egui::Color32, border: egui::Color32) {
    Frame::none()
        .fill(card).stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(16.0)).rounding(Rounding::same(8.0))
        .show(ui, |ui| {
            ui.set_min_width(120.0);
            ui.label(RichText::new(label).color(muted).size(12.0));
            ui.add_space(4.0);
            ui.label(RichText::new(value).color(fg).size(24.0).strong());
        });
}


pub fn sidebar_item(ui: &mut egui::Ui, label: &str, active: bool, fg: egui::Color32, primary: egui::Color32) -> bool {
    let color = if active { primary } else { fg };
    ui.add(egui::Button::new(RichText::new(label).color(color).size(14.0))
        .fill(egui::Color32::TRANSPARENT)
        .min_size(egui::vec2(156.0, 32.0))).clicked()
}


pub fn sidebar_item_with_badge(
    ui: &mut egui::Ui, label: &str, active: bool,
    fg: egui::Color32, primary: egui::Color32, badge: u64,
) -> bool {
    let color = if active { primary } else { fg };
    let btn_resp = ui.add(
        egui::Button::new(RichText::new(label).color(color).size(14.0))
            .fill(egui::Color32::TRANSPARENT)
            .min_size(egui::vec2(156.0, 32.0)),
    );
    if badge > 0 {
        let rect = btn_resp.rect;
        let bell_x = rect.left() + 18.0;
        let badge_center = egui::pos2(bell_x, rect.top() + 4.0);
        ui.painter().circle_filled(badge_center, 7.0, egui::Color32::from_rgb(239, 68, 68));
        ui.painter().text(
            badge_center,
            egui::Align2::CENTER_CENTER,
            format!("{}", badge.min(99)),
            egui::FontId::proportional(8.0),
            egui::Color32::WHITE,
        );
    }
    btn_resp.clicked()
}