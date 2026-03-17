use eframe::egui::{self, Color32, Frame, Margin, RichText, Rounding, ScrollArea, Stroke, Vec2};
use crate::state::{AppState, Screen};
use crate::screens::screenDashboard::sidebar_item;
use crate::screens::screenDashboard::sidebar_item_with_badge;

pub fn profile_screen(ctx: &egui::Context, state: &mut AppState) {
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
    let secondary      = state.theme.secondary;
    let secondary_fg   = state.theme.secondary_foreground;

    // ── TOP BAR ──────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("profile_top")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("👤 Profile").color(fg).size(18.0).strong());
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
    egui::SidePanel::left("profile_sidebar")
        .show_separator_line(false)
        .min_width(180.0).max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);
            if sidebar_item(ui, "📊 Dashboard",    false, fg, primary) { state.go_to(Screen::Dashboard); }
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
            sidebar_item(ui, "👤 Profile", true, fg, primary);
        });

    // ── CENTRAL PANEL ─────────────────────────────────────────────────────────
    egui::CentralPanel::default()
        .frame(Frame::none().fill(bg).inner_margin(Margin { left: 32.0, right: 56.0, top: 0.0, bottom: 0.0 }))
        .show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                ui.add_space(24.0);
                let width = ui.available_width().min(800.0);
                ui.vertical(|ui| {
                    ui.set_max_width(width);

                    // Header
                    ui.label(RichText::new("Account & Profile").color(fg).size(22.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Manage your account information and preferences").color(muted).size(13.0));
                    ui.add_space(24.0);

                    // ── Toast messages ─────────────────────────────────────
                    if let Some(msg) = state.profile_state.success_msg.clone() {
                        if state.profile_state.msg_time.elapsed().as_secs() < 5 {
                            Frame::none()
                                .fill(Color32::from_rgb(30, 55, 30))
                                .stroke(Stroke::new(1.0, chart_2))
                                .inner_margin(Margin::symmetric(16.0, 10.0))
                                .rounding(Rounding::same(8.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(&msg).color(chart_2).size(13.0));
                                });
                            ui.add_space(12.0);
                        } else {
                            state.profile_state.success_msg = None;
                        }
                    }
                    if let Some(err) = state.profile_state.error_msg.clone() {
                        Frame::none()
                            .fill(Color32::from_rgb(55, 22, 22))
                            .stroke(Stroke::new(1.0, destructive))
                            .inner_margin(Margin::symmetric(16.0, 10.0))
                            .rounding(Rounding::same(8.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!("⚠ {}", err)).color(destructive_fg).size(13.0));
                            });
                        ui.add_space(12.0);
                    }

                    // ── Profile Card ───────────────────────────────────────
                    section_card(ui, card, border, |ui| {
                        ui.horizontal(|ui| {
                            // Avatar circle
                            let avatar_size = 64.0;
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::splat(avatar_size), egui::Sense::hover(),
                            );
                            let user_initial = state.current_user.as_ref()
                                .and_then(|u| u.name.chars().next())
                                .unwrap_or('?')
                                .to_uppercase()
                                .next()
                                .unwrap_or('?');
                            ui.painter().circle_filled(
                                rect.center(), avatar_size / 2.0,
                                Color32::from_rgb(124, 58, 202),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                user_initial.to_string(),
                                egui::FontId::proportional(28.0),
                                Color32::WHITE,
                            );
                            ui.add_space(16.0);

                            ui.vertical(|ui| {
                                if let Some(user) = &state.current_user {
                                    ui.label(RichText::new(&user.name).color(fg).size(20.0).strong());
                                    ui.add_space(2.0);
                                    ui.label(RichText::new(&user.email).color(muted).size(13.0));
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        role_badge(ui, &user.role);
                                        ui.add_space(6.0);
                                        plan_badge(ui, state.billing_state.current_plan.name());
                                    });
                                    ui.add_space(4.0);
                                    let joined = user.created_at.format("%B %d, %Y").to_string();
                                    ui.label(RichText::new(format!("Joined {}", joined))
                                        .color(muted).size(12.0));
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                if !state.profile_state.is_editing {
                                    if ui.add(egui::Button::new(
                                        RichText::new("✏ Edit Profile").color(primary_fg).size(13.0)
                                    ).fill(primary).min_size(Vec2::new(110.0, 32.0))).clicked() {
                                        state.start_edit_profile();
                                    }
                                }
                            });
                        });

                        // ── Edit Form ─────────────────────────────────────
                        if state.profile_state.is_editing {
                            ui.add_space(16.0);
                            ui.painter().hline(
                                ui.available_rect_before_wrap().x_range(),
                                ui.cursor().top(),
                                Stroke::new(1.0, border),
                            );
                            ui.add_space(16.0);

                            ui.label(RichText::new("Edit Profile").color(fg).size(15.0).strong());
                            ui.add_space(12.0);

                            ui.label(RichText::new("Display Name").color(muted).size(12.0));
                            ui.add_space(4.0);
                            ui.add(egui::TextEdit::singleline(&mut state.profile_state.edit_name)
                                .hint_text("Your name")
                                .desired_width(280.0));
                            ui.add_space(10.0);

                            ui.label(RichText::new("Email").color(muted).size(12.0));
                            ui.add_space(4.0);
                            ui.add(egui::TextEdit::singleline(&mut state.profile_state.edit_email)
                                .hint_text("email@example.com")
                                .desired_width(280.0));
                            ui.add_space(4.0);
                            ui.label(RichText::new("Email changes require re-authentication (feature coming soon)")
                                .color(muted).size(11.0));
                            ui.add_space(16.0);

                            ui.horizontal(|ui| {
                                if ui.add(egui::Button::new(
                                    RichText::new("Save Changes").color(primary_fg).size(13.0)
                                ).fill(primary).min_size(Vec2::new(120.0, 32.0))).clicked() {
                                    state.save_profile_sync();
                                }
                                ui.add_space(8.0);
                                if ui.add(egui::Button::new(
                                    RichText::new("Cancel").color(fg).size(13.0)
                                ).fill(secondary).min_size(Vec2::new(80.0, 32.0))).clicked() {
                                    state.profile_state.is_editing = false;
                                    state.profile_state.error_msg  = None;
                                }
                            });
                        }
                    });

                    ui.add_space(24.0);

                    // ── Account Details ────────────────────────────────────
                    ui.label(RichText::new("Account Details").color(fg).size(16.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Your account information at a glance").color(muted).size(13.0));
                    ui.add_space(12.0);

                    section_card(ui, card, border, |ui| {
                        ui.set_max_width(width);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Role").color(muted).size(12.0));
                                ui.add_space(4.0);
                                ui.label(RichText::new(
                                    state.current_user.as_ref().map(|u| u.role.as_str()).unwrap_or("user")
                                ).color(fg).size(14.0));
                            });
                        });
                        ui.add_space(8.0);
                        ui.columns(2, |cols| {
                            info_row(&mut cols[0], "Subscription Plan",
                                state.billing_state.current_plan.name(), fg, muted);
                            info_row(&mut cols[1], "Account Status", "Active ✅", fg, muted);
                        });
                        ui.add_space(8.0);
                        ui.columns(2, |cols| {
                            let project_count = state.projects.len().to_string();
                            info_row(&mut cols[0], "Projects", &project_count, fg, muted);
                            let task_count = state.todo_state.items.len().to_string();
                            info_row(&mut cols[1], "To-Do Items", &task_count, fg, muted);
                        });
                    });

                    ui.add_space(24.0);

                    // ── Role & Permissions ─────────────────────────────────
                    let is_admin = state.current_user.as_ref()
                        .map(|u| u.role == "admin")
                        .unwrap_or(false);

                    ui.label(RichText::new("Role & Permissions").color(fg).size(16.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Your access level and capabilities").color(muted).size(13.0));
                    ui.add_space(12.0);

                    section_card(ui, card, border, |ui| {
                        ui.set_max_width(width);

                        permission_row(ui, "📁 Create & manage projects", true, fg, muted, chart_2);
                        ui.add_space(4.0);
                        permission_row(ui, "✅ Create & manage tasks", true, fg, muted, chart_2);
                        ui.add_space(4.0);
                        permission_row(ui, "💳 Manage billing & subscription", true, fg, muted, chart_2);
                        ui.add_space(4.0);
                        permission_row(ui, "🔔 Receive notifications", true, fg, muted, chart_2);
                        ui.add_space(4.0);
                        permission_row(ui, "👥 Manage all users (Admin)", is_admin, fg, muted, chart_2);
                        ui.add_space(4.0);
                        permission_row(ui, "⚙ System configuration (Admin)", is_admin, fg, muted, chart_2);

                        if is_admin {
                            ui.add_space(16.0);
                            ui.painter().hline(
                                ui.available_rect_before_wrap().x_range(),
                                ui.cursor().top(),
                                Stroke::new(1.0, border),
                            );
                            ui.add_space(16.0);
                            ui.label(RichText::new("⚡ Admin Panel").color(Color32::from_rgb(245, 158, 11)).size(14.0).strong());
                            ui.add_space(8.0);
                            ui.label(RichText::new("You have administrator access to this platform.")
                                .color(muted).size(12.0));
                            ui.add_space(8.0);
                            Frame::none()
                                .fill(Color32::from_rgb(50, 45, 20))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(245, 158, 11)))
                                .inner_margin(Margin::symmetric(14.0, 8.0))
                                .rounding(Rounding::same(8.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("⚠ Admin features (user management, audit logs) are available via the API.")
                                        .color(Color32::from_rgb(245, 158, 11)).size(12.0));
                                });
                        }
                    });

                    ui.add_space(24.0);

                    // ── Account Settings ───────────────────────────────────
                    ui.label(RichText::new("Account Settings").color(fg).size(16.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Manage your preferences").color(muted).size(13.0));
                    ui.add_space(12.0);

                    section_card(ui, card, border, |ui| {
                        ui.set_max_width(width);
                        setting_row(ui, "🎨 Theme", "Dark mode (current)", secondary, secondary_fg, fg, muted, || {});
                        ui.add_space(4.0);
                        setting_row(ui, "🔔 Notifications", "Enabled", secondary, secondary_fg, fg, muted, || {});
                        ui.add_space(4.0);
                        setting_row(ui, "🔒 Two-Factor Auth", "Not configured", secondary, secondary_fg, fg, muted, || {});
                    });

                    ui.add_space(24.0);

                    // ── Danger Zone ────────────────────────────────────────
                    ui.label(RichText::new("Danger Zone").color(destructive_fg).size(16.0).strong());
                    ui.add_space(12.0);

                    Frame::none()
                        .fill(Color32::from_rgb(45, 28, 28))
                        .stroke(Stroke::new(1.0, destructive))
                        .inner_margin(Margin::same(20.0))
                        .rounding(Rounding::same(12.0))
                        .show(ui, |ui| {
                            ui.set_max_width(width);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Sign out").color(fg).size(14.0).strong());
                                    ui.add_space(2.0);
                                    ui.label(RichText::new("Sign out from all devices and return to login")
                                        .color(muted).size(12.0));
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add(egui::Button::new(
                                        RichText::new("🔓 Sign Out").color(destructive_fg).size(13.0)
                                    ).fill(destructive).min_size(Vec2::new(100.0, 32.0))).clicked() {
                                        state.logout();
                                    }
                                });
                            });
                        });

                    ui.add_space(32.0);
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  HELPERS
// ─────────────────────────────────────────────────────────────────────────────

fn section_card(ui: &mut egui::Ui, card: Color32, border: Color32, content: impl FnOnce(&mut egui::Ui)) {
    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(12.0))
        .show(ui, content);
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str, fg: Color32, muted: Color32) {
    ui.label(RichText::new(label).color(muted).size(11.0));
    ui.add_space(2.0);
    ui.label(RichText::new(value).color(fg).size(13.0).strong());
}

fn permission_row(
    ui: &mut egui::Ui,
    label: &str,
    granted: bool,
    fg: Color32, muted: Color32, chart_2: Color32,
) {
    ui.horizontal(|ui| {
        let (icon, color) = if granted { ("✅", chart_2) } else { ("🔒", muted) };
        ui.label(RichText::new(icon).size(14.0));
        ui.add_space(8.0);
        ui.label(RichText::new(label).color(if granted { fg } else { muted }).size(13.0));
        if !granted {
            ui.add_space(6.0);
            Frame::none()
                .fill(Color32::from_rgb(60, 60, 60))
                .inner_margin(Margin::symmetric(6.0, 2.0))
                .rounding(Rounding::same(20.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Requires Admin").color(muted).size(10.0));
                });
        }
    });
}

fn setting_row(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    secondary: Color32, secondary_fg: Color32,
    fg: Color32, muted: Color32,
    _on_click: impl FnOnce(),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_min_width(200.0);
            ui.label(RichText::new(label).color(fg).size(13.0));
            ui.add_space(1.0);
            ui.label(RichText::new(value).color(muted).size(11.0));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Button::new(
                RichText::new("Configure").color(secondary_fg).size(11.0)
            ).fill(secondary).min_size(Vec2::new(80.0, 26.0)));
        });
    });
}

fn role_badge(ui: &mut egui::Ui, role: &str) {
    let (fill, text) = match role {
        "admin" => (Color32::from_rgb(124, 58, 202), "⭐ Admin"),
        _       => (Color32::from_rgb(59, 130, 246), "👤 Member"),
    };
    Frame::none()
        .fill(fill)
        .inner_margin(Margin::symmetric(8.0, 3.0))
        .rounding(Rounding::same(20.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(Color32::WHITE).size(11.0).strong());
        });
}

fn plan_badge(ui: &mut egui::Ui, plan: &str) {
    let fill = match plan {
        "Pro"        => Color32::from_rgb(16, 130, 80),
        "Enterprise" => Color32::from_rgb(200, 120, 10),
        "Starter"    => Color32::from_rgb(59, 100, 200),
        _            => Color32::from_rgb(80, 80, 80),
    };
    Frame::none()
        .fill(fill)
        .inner_margin(Margin::symmetric(8.0, 3.0))
        .rounding(Rounding::same(20.0))
        .show(ui, |ui| {
            ui.label(RichText::new(format!("💳 {}", plan))
                .color(Color32::WHITE).size(11.0));
        });
}