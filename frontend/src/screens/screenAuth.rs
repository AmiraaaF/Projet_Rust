use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke};
use crate::state::{AppState, Screen};
use shared::models::*;

pub fn login_screen(ctx: &egui::Context, state: &mut AppState) {
    let bg = state.theme.background;
    let card = state.theme.card;
    let fg = state.theme.foreground;
    let muted = state.theme.muted_foreground;
    let border = state.theme.border;
    let primary = state.theme.primary;
    let primary_fg = state.theme.primary_foreground;
    let destructive = state.theme.destructive;
    let secondary = state.theme.secondary;
    let secondary_fg = state.theme.secondary_foreground;
    let chart_2 = state.theme.chart_2;

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);

                Frame::none()
                    .fill(card)
                    .stroke(Stroke::new(1.0, border))
                    .inner_margin(Margin::same(32.0))
                    .rounding(Rounding::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(320.0);

                        ui.label(RichText::new("🔐 Connexion").color(fg).size(22.0).strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new("Bienvenue sur Mini-SaaS").color(muted).size(13.0));
                        ui.add_space(20.0);

                        ui.label(RichText::new("Email").color(fg).size(13.0));
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.email_input)
                                .hint_text("votre@email.com")
                                .desired_width(280.0)
                        );
                        ui.add_space(12.0);

                        ui.label(RichText::new("Mot de passe").color(fg).size(13.0));
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.password_input)
                                .hint_text("••••••••")
                                .password(true)
                                .desired_width(280.0)
                        );
                        ui.add_space(20.0);

                        let login_clicked = ui.add(
                            egui::Button::new(
                                RichText::new("Se connecter").color(primary_fg).size(14.0)
                            )
                            .fill(primary)
                            .min_size(egui::vec2(280.0, 36.0))
                        ).clicked();

                        ui.add_space(8.0);

                        let reg_clicked = ui.add(
                            egui::Button::new(
                                RichText::new("Créer un compte").color(secondary_fg).size(13.0)
                            )
                            .fill(secondary)
                            .min_size(egui::vec2(280.0, 32.0))
                        ).clicked();

                        ui.add_space(12.0);

                        if let Some(error) = &state.error_message.clone() {
                            ui.label(RichText::new(format!("⚠ {}", error)).color(destructive).size(12.0));
                        }
                        if let Some(success) = &state.success_message.clone() {
                            ui.label(RichText::new(format!("✓ {}", success)).color(chart_2).size(12.0));
                        }

                        // Actions après affichage
                        if login_clicked {
                            let email = state.email_input.clone();
                            let password = state.password_input.clone();
                            if !email.is_empty() && !password.is_empty() {
                                state.current_user = Some(UserPublic {
                                    id: uuid::Uuid::new_v4(),
                                    email,
                                    name: "User".to_string(),
                                    role: "user".to_string(),
                                    created_at: chrono::Utc::now(),
                                });
                                state.go_to(Screen::Dashboard); // vide les messages automatiquement
                            } else {
                                state.error_message = Some("Veuillez remplir tous les champs".to_string());
                            }
                        }

                        if reg_clicked {
                            state.clear_forms();
                            state.go_to(Screen::Register); // vide les messages automatiquement
                        }
                    });
            });
        });
}

pub fn register_screen(ctx: &egui::Context, state: &mut AppState) {
    let bg = state.theme.background;
    let card = state.theme.card;
    let fg = state.theme.foreground;
    let muted = state.theme.muted_foreground;
    let border = state.theme.border;
    let primary = state.theme.primary;
    let primary_fg = state.theme.primary_foreground;
    let destructive = state.theme.destructive;
    let secondary = state.theme.secondary;
    let secondary_fg = state.theme.secondary_foreground;
    let chart_2 = state.theme.chart_2;

    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);

                Frame::none()
                    .fill(card)
                    .stroke(Stroke::new(1.0, border))
                    .inner_margin(Margin::same(32.0))
                    .rounding(Rounding::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(320.0);

                        ui.label(RichText::new("📝 Créer un compte").color(fg).size(22.0).strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new("Rejoignez Mini-SaaS").color(muted).size(13.0));
                        ui.add_space(20.0);

                        ui.label(RichText::new("Nom").color(fg).size(13.0));
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.name_input)
                                .hint_text("Votre nom")
                                .desired_width(280.0)
                        );
                        ui.add_space(12.0);

                        ui.label(RichText::new("Email").color(fg).size(13.0));
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.email_input)
                                .hint_text("votre@email.com")
                                .desired_width(280.0)
                        );
                        ui.add_space(12.0);

                        ui.label(RichText::new("Mot de passe").color(fg).size(13.0));
                        ui.add_space(4.0);
                        ui.add(
                            egui::TextEdit::singleline(&mut state.password_input)
                                .hint_text("••••••••")
                                .password(true)
                                .desired_width(280.0)
                        );
                        ui.add_space(20.0);

                        let reg_clicked = ui.add(
                            egui::Button::new(
                                RichText::new("S'inscrire").color(primary_fg).size(14.0)
                            )
                            .fill(primary)
                            .min_size(egui::vec2(280.0, 36.0))
                        ).clicked();

                        ui.add_space(8.0);

                        let back_clicked = ui.add(
                            egui::Button::new(
                                RichText::new("Retour").color(secondary_fg).size(13.0)
                            )
                            .fill(secondary)
                            .min_size(egui::vec2(280.0, 32.0))
                        ).clicked();

                        ui.add_space(12.0);

                        if let Some(error) = &state.error_message.clone() {
                            ui.label(RichText::new(format!(" {}", error)).color(destructive).size(12.0));
                        }
                        if let Some(success) = &state.success_message.clone() {
                            ui.label(RichText::new(format!(" {}", success)).color(chart_2).size(12.0));
                        }

                        if reg_clicked {
                            if !state.name_input.is_empty()
                                && !state.email_input.is_empty()
                                && !state.password_input.is_empty()
                            {
                                // On met le message AVANT go_to car go_to vide les messages
                                // Donc on va sur Login et le message sera visible sur login
                                state.clear_forms();
                                // On met d'abord le message puis on change d'écran
                                // go_to vide les messages donc on fait autrement ici :
                                state.current_screen = Screen::Login;
                                state.error_message = None;
                                state.success_message = Some("Inscription réussie ! Connectez-vous.".to_string());
                            } else {
                                state.error_message = Some("Veuillez remplir tous les champs".to_string());
                                state.success_message = None;
                            }
                        }

                        if back_clicked {
                            state.clear_forms();
                            state.go_to(Screen::Login);
                        }
                    });
            });
        });
}