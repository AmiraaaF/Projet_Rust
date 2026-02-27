use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke};
use crate::state::{AppState, Screen};

pub fn dashboard_screen(ctx: &egui::Context, state: &mut AppState) {
    let bg = state.theme.background;
    let sidebar_bg = state.theme.sidebar;
    let fg = state.theme.foreground;
    let muted = state.theme.muted_foreground;
    let border = state.theme.border;
    let card = state.theme.card;
    let primary = state.theme.primary;
    let primary_fg = state.theme.primary_foreground;
    let destructive = state.theme.destructive;
    let destructive_fg = state.theme.destructive_foreground;

    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📊 Dashboard").color(fg).size(18.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let logout_btn = egui::Button::new(
                        RichText::new("🔓 Déconnexion").color(destructive_fg).size(13.0)
                    ).fill(destructive);
                    if ui.add(logout_btn).clicked() {
                        state.logout();
                    }
                    if let Some(user) = &state.current_user.clone() {
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

            sidebar_item(ui, "📊 Dashboard", true, fg, primary);
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projets", false, fg, primary) {
                state.go_to(Screen::Projects);
            }
        });

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg).inner_margin(Margin::same(24.0)))
        .show(ctx, |ui| {
            ui.label(RichText::new("Vue d'ensemble").color(fg).size(20.0).strong());
            ui.add_space(4.0);
            ui.label(RichText::new("Bienvenue sur votre tableau de bord").color(muted).size(13.0));
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                stat_card(ui, "📁 Projets", &state.projects.len().to_string(), card, fg, muted, border);
                ui.add_space(12.0);
                stat_card(ui, "✅ Tâches", &state.current_tasks.len().to_string(), card, fg, muted, border);
                ui.add_space(12.0);
                stat_card(ui, "👥 Membres", "1", card, fg, muted, border);
            });

            ui.add_space(24.0);

            ui.label(RichText::new("Projets récents").color(fg).size(16.0).strong());
            ui.add_space(12.0);

            if state.projects.is_empty() {
                Frame::none()
                    .fill(card)
                    .stroke(Stroke::new(1.0, border))
                    .inner_margin(Margin::same(20.0))
                    .rounding(Rounding::same(8.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("Aucun projet pour l'instant").color(muted).size(13.0));
                            ui.add_space(8.0);
                            let btn = egui::Button::new(
                                RichText::new("+ Créer un projet").color(primary_fg)
                            ).fill(primary);
                            if ui.add(btn).clicked() {
                                state.go_to(Screen::Projects);
                            }
                        });
                    });
            } else {
                let projects_clone = state.projects.clone();
                for project in projects_clone.iter().take(5) {
                    Frame::none()
                        .fill(card)
                        .stroke(Stroke::new(1.0, border))
                        .inner_margin(Margin::symmetric(16.0, 10.0))
                        .rounding(Rounding::same(8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("📁 {}", project.name)).color(fg));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let btn = egui::Button::new(
                                        RichText::new("Voir").color(primary_fg).size(12.0)
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

fn stat_card(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    card: egui::Color32,
    fg: egui::Color32,
    muted: egui::Color32,
    border: egui::Color32,
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
            ui.label(RichText::new(value).color(fg).size(28.0).strong());
        });
}

fn sidebar_item(ui: &mut egui::Ui, label: &str, active: bool, fg: egui::Color32, primary: egui::Color32) -> bool {
    let color = if active { primary } else { fg };
    let btn = egui::Button::new(RichText::new(label).color(color).size(14.0))
        .fill(egui::Color32::TRANSPARENT)
        .min_size(egui::vec2(156.0, 32.0));
    ui.add(btn).clicked()
}