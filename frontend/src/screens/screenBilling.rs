use eframe::egui::{self, RichText, Frame, Margin, Rounding, Stroke, ScrollArea, Color32, Vec2};
use crate::state::{AppState, Screen};

// ─────────────────────────────────────────────────────────────────────────────
//  PLAN
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    Free,
    Starter,
    Pro,
    Enterprise,
}

impl Plan {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "starter"    => Plan::Starter,
            "pro"        => Plan::Pro,
            "enterprise" => Plan::Enterprise,
            _            => Plan::Free,
        }
    }

    pub fn api_name(&self) -> &str {
        match self {
            Plan::Free       => "free",
            Plan::Starter    => "starter",
            Plan::Pro        => "pro",
            Plan::Enterprise => "enterprise",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Plan::Free       => "Free",
            Plan::Starter    => "Starter",
            Plan::Pro        => "Pro",
            Plan::Enterprise => "Enterprise",
        }
    }

    pub fn price(&self) -> &str {
        match self {
            Plan::Free       => "$0/forever",
            Plan::Starter    => "$9.99/month",
            Plan::Pro        => "$29.99/month",
            Plan::Enterprise => "$99.99/month",
        }
    }

    pub fn price_amount(&self) -> f64 {
        match self {
            Plan::Free       => 0.0,
            Plan::Starter    => 9.99,
            Plan::Pro        => 29.99,
            Plan::Enterprise => 99.99,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Plan::Free       => "For individuals getting started",
            Plan::Starter    => "For small teams with basic needs",
            Plan::Pro        => "For growing teams that need more power",
            Plan::Enterprise => "For large organizations with custom needs",
        }
    }

    pub fn features(&self) -> Vec<&str> {
        match self {
            Plan::Free => vec![
                "3 projects",
                "100 tasks",
                "Community support",
                "Basic API access",
            ],
            Plan::Starter => vec![
                "10 projects",
                "500 tasks",
                "Email support",
                "Basic API access",
            ],
            Plan::Pro => vec![
                "50 projects",
                "5,000 tasks",
                "Priority support",
                "Full API access",
                "Advanced exports",
                "Advanced analytics",
            ],
            Plan::Enterprise => vec![
                "Unlimited projects",
                "Unlimited tasks",
                "24/7 dedicated support",
                "Full API access",
                "SLA 99.9% guarantee",
                "Custom onboarding",
            ],
        }
    }

    pub fn max_projects(&self) -> &str {
        match self {
            Plan::Free       => "3",
            Plan::Starter    => "10",
            Plan::Pro        => "50",
            Plan::Enterprise => "∞",
        }
    }

    pub fn max_tasks(&self) -> &str {
        match self {
            Plan::Free       => "100",
            Plan::Starter    => "500",
            Plan::Pro        => "5,000",
            Plan::Enterprise => "∞",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  INVOICE 
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BillingInvoice {
    pub id: String,
    pub date: String,
    pub plan: String,
    pub amount: f64,
    pub currency: String,
    pub status: InvoiceStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvoiceStatus {
    Paid,
    Pending,
    Overdue,
}

impl InvoiceStatus {
    pub fn label(&self) -> &str {
        match self {
            InvoiceStatus::Paid    => "Paid",
            InvoiceStatus::Pending => "Pending",
            InvoiceStatus::Overdue => "Overdue",
        }
    }
    pub fn color(&self) -> Color32 {
        match self {
            InvoiceStatus::Paid    => Color32::from_rgb(132, 204, 22),
            InvoiceStatus::Pending => Color32::from_rgb(245, 158, 11),
            InvoiceStatus::Overdue => Color32::from_rgb(239, 68, 68),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  MAIN SCREEN
// ─────────────────────────────────────────────────────────────────────────────

pub fn billing_screen(ctx: &egui::Context, state: &mut AppState) {
    state.load_invoices_sync();

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

    // ── TOP BAR ──────────────────────────────────────────────────────────────
    egui::TopBottomPanel::top("billing_top_panel")
        .show_separator_line(false)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::symmetric(16.0, 10.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("💳 Billing").color(fg).size(18.0).strong());
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
    egui::SidePanel::left("billing_sidebar")
        .show_separator_line(false)
        .min_width(180.0).max_width(180.0)
        .frame(Frame::none().fill(sidebar_bg).inner_margin(Margin::same(12.0)))
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new("NAVIGATION").color(muted).size(11.0));
            ui.add_space(8.0);
            if sidebar_item(ui, "📊 Dashboard", false, fg, primary) { state.go_to(Screen::Dashboard); }
            ui.add_space(4.0);
            if sidebar_item(ui, "📁 Projects",  false, fg, primary) { state.go_to(Screen::Projects); }
            ui.add_space(4.0);
            sidebar_item(ui, "💳 Billing", true, fg, primary);
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

                    // ── Section 1: current plan ────────────────────────────
                    section_header(ui, "Billing", "Manage your subscription and billing information", fg, muted);
                    ui.add_space(16.0);
                    current_subscription_card(ui, state, card, fg, muted, border, chart_2);
                    ui.add_space(32.0);

                    // ── Section 2: plan cards ──────────────────────────────
                    ui.label(RichText::new("Choose your plan").color(fg).size(18.0).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new("Select the perfect plan for your team").color(muted).size(13.0));
                    ui.add_space(16.0);
                    plan_cards(ui, state, content_width, card, fg, muted, border, primary, primary_fg, chart_2);
                    ui.add_space(32.0);

                    // ── Section 3: usage ───────────────────────────────────
                    usage_section(ui, state, content_width, card, fg, muted, border, chart_2, primary);
                    ui.add_space(32.0);

                    // ── Section 4: invoices ────────────────────────────────
                    invoice_section(ui, state, content_width, card, fg, muted, border, chart_2, primary, primary_fg);
                    ui.add_space(32.0);
                });
            });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  CURRENT SUBSCRIPTION CARD
// ─────────────────────────────────────────────────────────────────────────────

fn current_subscription_card(
    ui: &mut egui::Ui,
    state: &mut AppState,
    card: Color32, fg: Color32, muted: Color32, border: Color32, _chart_2: Color32,
) {
    let current_plan = state.billing_state.current_plan.clone();

    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(20.0))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());

            if let Some(err) = state.billing_state.last_error.clone() {
                Frame::none()
                    .fill(Color32::from_rgb(55, 22, 22))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                    .inner_margin(Margin::symmetric(14.0, 8.0))
                    .rounding(Rounding::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("⚠ {}", err))
                                .color(Color32::from_rgb(239, 68, 68)).size(12.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("[X]").clicked() {
                                    state.billing_state.last_error = None;
                                }
                            });
                        });
                    });
                ui.add_space(12.0);
            }

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    // Badge plan
                    Frame::none()
                        .fill(Color32::from_rgb(124, 58, 202))
                        .inner_margin(Margin::symmetric(10.0, 4.0))
                        .rounding(Rounding::same(20.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(format!("{} Plan", current_plan.name()))
                                .color(Color32::WHITE).size(12.0).strong());
                        });
                    ui.add_space(8.0);
                    if current_plan != Plan::Free {
                        ui.label(RichText::new("Your subscription is active").color(muted).size(13.0));
                    } else {
                        ui.label(RichText::new("You are on the free plan").color(muted).size(13.0));
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new(current_plan.price()).color(fg).size(22.0).strong());
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if current_plan != Plan::Free {
                        let cancel_btn = egui::Button::new(
                            RichText::new("Cancel Subscription").color(Color32::from_rgb(239, 68, 68)).size(13.0),
                        )
                        .fill(Color32::from_rgb(60, 30, 30))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                        .min_size(Vec2::new(160.0, 34.0));
                        if ui.add(cancel_btn).clicked() {
                            state.billing_state.show_cancel_confirm = true;
                        }
                    }
                });
            });

            // Cancel confirmation
            if state.billing_state.show_cancel_confirm {
                ui.add_space(12.0);
                Frame::none()
                    .fill(Color32::from_rgb(60, 30, 30))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                    .inner_margin(Margin::same(14.0))
                    .rounding(Rounding::same(8.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("⚠ Are you sure you want to cancel your subscription?")
                            .color(Color32::from_rgb(239, 68, 68)).size(13.0));
                        ui.add_space(4.0);
                        ui.label(RichText::new("You will lose access to premium features immediately.")
                            .color(muted).size(12.0));
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            let confirm = egui::Button::new(RichText::new("Yes, Cancel").color(Color32::WHITE).size(13.0))
                                .fill(Color32::from_rgb(180, 50, 50))
                                .min_size(Vec2::new(120.0, 32.0));
                            if ui.add(confirm).clicked() {
                                match state.cancel_subscription_sync() {
                                    Ok(_) => {
                                        state.billing_state.show_cancel_confirm = false;
                                        state.billing_state.invoices_loaded = false; 
                                    }
                                    Err(e) => {
                                        state.billing_state.last_error = Some(e);
                                        state.billing_state.show_cancel_confirm = false;
                                    }
                                }
                            }
                            ui.add_space(8.0);
                            let keep = egui::Button::new(RichText::new("Keep Subscription").color(fg).size(13.0))
                                .fill(Color32::from_rgb(68, 68, 68))
                                .min_size(Vec2::new(140.0, 32.0));
                            if ui.add(keep).clicked() {
                                state.billing_state.show_cancel_confirm = false;
                            }
                        });
                    });
            }
        });
}

// ─────────────────────────────────────────────────────────────────────────────
//  PLAN CARDS
// ─────────────────────────────────────────────────────────────────────────────

fn plan_cards(
    ui: &mut egui::Ui,
    state: &mut AppState,
    content_width: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    primary: Color32, primary_fg: Color32, chart_2: Color32,
) {
    let plans = [Plan::Free, Plan::Pro, Plan::Enterprise];

    let card_width = if content_width > 900.0 {
        (content_width - 20.0) / 3.0 - 4.0
    } else if content_width > 600.0 {
        (content_width - 12.0) / 2.0 - 4.0
    } else {
        content_width - 16.0
    };

    ui.horizontal_wrapped(|ui| {
        ui.set_max_width(content_width);
        for plan in &plans {
            let is_current = &state.billing_state.current_plan == plan;
            let is_popular = *plan == Plan::Pro;

            let card_fill = if is_current { Color32::from_rgb(52, 52, 65) } else { card };
            let card_border = if is_current { Color32::from_rgb(124, 58, 202) } else { border };

            Frame::none()
                .fill(card_fill)
                .stroke(Stroke::new(if is_current { 2.0 } else { 1.0 }, card_border))
                .inner_margin(Margin::same(8.0))
                .rounding(Rounding::same(12.0))
                .show(ui, |ui| {
                    ui.set_min_width(card_width);
                    ui.set_max_width(card_width);
                    ui.set_min_height(360.0);

                    ui.vertical(|ui| {
                        if is_popular {
                            ui.horizontal(|ui| {
                                Frame::none()
                                    .fill(Color32::from_rgb(124, 58, 202))
                                    .inner_margin(Margin::symmetric(8.0, 3.0))
                                    .rounding(Rounding::same(20.0))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new("⭐ Most Popular").color(Color32::WHITE).size(11.0).strong());
                                    });
                            });
                            ui.add_space(6.0);
                        } else {
                            ui.add_space(18.0);
                        }

                        ui.label(RichText::new(plan.name()).color(fg).size(18.0).strong());
                        ui.add_space(4.0);
                        ui.label(RichText::new(plan.description()).color(muted).size(12.0));
                        ui.add_space(12.0);
                        ui.label(RichText::new(plan.price()).color(fg).size(20.0).strong());
                        ui.add_space(12.0);

                        ui.painter().hline(
                            ui.available_rect_before_wrap().x_range(),
                            ui.cursor().top(),
                            Stroke::new(1.0, border),
                        );
                        ui.add_space(12.0);

                        for feature in plan.features() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("✅").color(chart_2).size(12.0));
                                ui.add_space(4.0);
                                ui.label(RichText::new(feature).color(fg).size(12.0));
                            });
                            ui.add_space(3.0);
                        }

                        ui.add_space(12.0);

                        if is_current {
                            let btn = egui::Button::new(
                                RichText::new("✅ Current Plan").color(Color32::WHITE).size(12.0).strong(),
                            )
                            .fill(Color32::from_rgb(59, 130, 246))
                            .min_size(Vec2::new(card_width - 16.0, 40.0));
                            ui.add_enabled(false, btn);
                        } else {
                            let label = match plan {
                                Plan::Free       => "Downgrade to Free",
                                Plan::Starter    => "Upgrade to Starter",
                                Plan::Pro        => "Upgrade to Pro",
                                Plan::Enterprise => "Upgrade to Enterprise",
                            };
                            let btn = egui::Button::new(
                                RichText::new(label).color(Color32::WHITE).size(12.0).strong(),
                            )
                            .fill(Color32::from_rgb(59, 130, 246))
                            .min_size(Vec2::new(card_width - 16.0, 40.0));
                            if ui.add(btn).clicked() {
                                state.billing_state.pending_plan = Some(plan.clone());
                                state.billing_state.show_upgrade_confirm = true;
                            }
                        }
                    });
                });
            ui.add_space(4.0);
        }
    });

    // Confirmation modal
    if state.billing_state.show_upgrade_confirm {
        if let Some(pending) = state.billing_state.pending_plan.clone() {
            ui.add_space(16.0);
            Frame::none()
                .fill(Color32::from_rgb(45, 45, 58))
                .stroke(Stroke::new(1.5, Color32::from_rgb(124, 58, 202)))
                .inner_margin(Margin::same(20.0))
                .rounding(Rounding::same(12.0))
                .show(ui, |ui| {
                    let is_upgrade = state.billing_state.current_plan.price_amount() < pending.price_amount();
                    let title = if is_upgrade {
                        format!("Upgrade to {} Plan", pending.name())
                    } else {
                        format!("Downgrade to {} Plan", pending.name())
                    };
                    ui.label(RichText::new(title).color(fg).size(16.0).strong());
                    ui.add_space(8.0);
                    ui.label(RichText::new(format!(
                        "New plan: {} — {}",
                        pending.name(), pending.price()
                    )).color(muted).size(13.0));
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!(
                        "Limits: {} projects, {} tasks",
                        pending.max_projects(), pending.max_tasks()
                    )).color(muted).size(12.0));
                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        let confirm = egui::Button::new(RichText::new("Confirm").color(primary_fg).size(13.0))
                            .fill(primary)
                            .min_size(Vec2::new(100.0, 32.0));
                        if ui.add(confirm).clicked() {
                            match state.update_plan_sync(&pending) {
                                Ok(confirmed_plan) => {
                                    state.billing_state.current_plan = Plan::from_str(&confirmed_plan);
                                    state.billing_state.pending_plan = None;
                                    state.billing_state.show_upgrade_confirm = false;
                                    state.billing_state.last_error = None;
                                    state.billing_state.invoices_loaded = false; 
                                }
                                Err(e) => {
                                    state.billing_state.last_error = Some(format!("Erreur: {}", e));
                                    state.billing_state.pending_plan = None;
                                    state.billing_state.show_upgrade_confirm = false;
                                }
                            }
                        }
                        ui.add_space(8.0);
                        let cancel = egui::Button::new(RichText::new("Cancel").color(fg).size(13.0))
                            .fill(Color32::from_rgb(68, 68, 68))
                            .min_size(Vec2::new(100.0, 32.0));
                        if ui.add(cancel).clicked() {
                            state.billing_state.pending_plan = None;
                            state.billing_state.show_upgrade_confirm = false;
                        }
                    });
                });
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  USAGE SECTION
// ─────────────────────────────────────────────────────────────────────────────

fn usage_section(
    ui: &mut egui::Ui, state: &AppState, content_width: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    chart_2: Color32, primary: Color32,
) {
    let plan = &state.billing_state.current_plan;
    let proj_used = state.projects.len();
    let proj_max: Option<usize> = match plan {
        Plan::Free    => Some(3),
        Plan::Starter => Some(10),
        Plan::Pro     => Some(50),
        Plan::Enterprise => None,
    };
    let tasks_max: Option<usize> = match plan {
        Plan::Free    => Some(100),
        Plan::Starter => Some(500),
        Plan::Pro     => Some(5000),
        Plan::Enterprise => None,
    };

    ui.label(RichText::new("Usage").color(fg).size(18.0).strong());
    ui.add_space(4.0);
    ui.label(RichText::new("Your current resource usage").color(muted).size(13.0));
    ui.add_space(12.0);

    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .inner_margin(Margin::same(16.0))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);
            usage_bar(ui, "Projects", proj_used, proj_max, fg, muted, chart_2, primary);
            ui.add_space(14.0);
            usage_bar(ui, "Tasks", state.current_tasks.len(), tasks_max, fg, muted, chart_2, primary);
        });
}

fn usage_bar(
    ui: &mut egui::Ui, label: &str, used: usize, max: Option<usize>,
    fg: Color32, muted: Color32, chart_2: Color32, _primary: Color32,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(fg).size(13.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let max_label = match max { Some(m) => format!("{} limit", m), None => "Unlimited".to_string() };
            ui.label(RichText::new(&max_label).color(muted).size(12.0));
            ui.label(RichText::new(" · ").color(muted).size(12.0));
            ui.label(RichText::new(format!("{} used", used)).color(fg).size(12.0));
        });
    });
    ui.add_space(4.0);
    let ratio = match max { Some(m) if m > 0 => (used as f32 / m as f32).min(1.0), _ => 0.0 };
    let bar_h = 6.0;
    let full_w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(full_w, bar_h), egui::Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(3.0), Color32::from_rgb(68, 68, 68));
    if max.is_none() {
        ui.painter().rect_filled(rect, Rounding::same(3.0), chart_2);
        ui.painter().text(egui::pos2(rect.center().x, rect.center().y),
            egui::Align2::CENTER_CENTER, "∞", egui::FontId::proportional(16.0), Color32::BLACK);
    } else if ratio > 0.0 {
        let fill_color = if ratio > 0.85 { Color32::from_rgb(239, 68, 68) }
                         else if ratio > 0.65 { Color32::from_rgb(245, 158, 11) }
                         else { chart_2 };
        let fr = egui::Rect::from_min_size(rect.min, Vec2::new(full_w * ratio, bar_h));
        ui.painter().rect_filled(fr, Rounding::same(3.0), fill_color);
    }
}

// ───────────────────
//  INVOICE SECTION 
// ────────────────────

fn invoice_section(
    ui: &mut egui::Ui, state: &mut AppState, content_width: f32,
    card: Color32, fg: Color32, muted: Color32, border: Color32,
    chart_2: Color32, primary: Color32, primary_fg: Color32,
) {
    ui.label(RichText::new("Invoice History").color(fg).size(18.0).strong());
    ui.add_space(4.0);
    ui.label(RichText::new("Your past invoices ").color(muted).size(13.0));
    ui.add_space(12.0);

    let invoices = state.billing_state.invoices.clone();

    Frame::none()
        .fill(card)
        .stroke(Stroke::new(1.0, border))
        .rounding(Rounding::same(12.0))
        .show(ui, |ui| {
            ui.set_max_width(content_width);

            // Header
            Frame::none()
                .fill(Color32::from_rgb(48, 48, 55))
                .inner_margin(Margin::symmetric(16.0, 12.0))
                .show(ui, |ui| {
                    ui.columns(5, |cols| {
                        cols[0].label(RichText::new("Invoice ID").color(muted).size(12.0).strong());
                        cols[1].label(RichText::new("Date").color(muted).size(12.0).strong());
                        cols[2].label(RichText::new("Amount").color(muted).size(12.0).strong());
                        cols[3].label(RichText::new("Status").color(muted).size(12.0).strong());
                        cols[4].label(RichText::new("Actions").color(muted).size(12.0).strong());
                    });
                });

            if invoices.is_empty() {
                Frame::none()
                    .inner_margin(Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No invoices yet").color(muted).size(13.0));
                            ui.add_space(4.0);
                            ui.label(RichText::new("Invoices will appear here when you upgrade to a paid plan")
                                .color(muted).size(12.0));
                        });
                    });
            } else {
                for (i, invoice) in invoices.iter().enumerate() {
                    let row_fill = if i % 2 == 0 { card } else { Color32::from_rgb(42, 42, 50) };
                    Frame::none()
                        .fill(row_fill)
                        .inner_margin(Margin::symmetric(16.0, 10.0))
                        .show(ui, |ui| {
                            ui.columns(5, |cols| {
                                cols[0].label(RichText::new(&invoice.id).color(fg).size(12.0).monospace());
                                cols[1].label(RichText::new(&invoice.date).color(muted).size(12.0));
                                cols[2].label(RichText::new(format!("${:.2} {}", invoice.amount, invoice.currency))
                                    .color(fg).size(12.0).strong());

                                
                                cols[3].horizontal(|ui| {
                                    Frame::none()
                                        .fill(Color32::from_rgba_premultiplied(
                                            invoice.status.color().r(),
                                            invoice.status.color().g(),
                                            invoice.status.color().b(),
                                            30,
                                        ))
                                        .stroke(Stroke::new(1.0, invoice.status.color()))
                                        .inner_margin(Margin::symmetric(6.0, 2.0))
                                        .rounding(Rounding::same(20.0))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new(invoice.status.label())
                                                .color(Color32::BLACK).size(11.0).strong());
                                        });
                                });

                                cols[4].horizontal(|ui| {
                                    let view_btn = egui::Button::new(
                                        RichText::new("View").color(primary_fg).size(11.0),
                                    ).fill(primary).min_size(Vec2::new(44.0, 22.0));
                                    if ui.add(view_btn).clicked() {
                                        state.billing_state.selected_invoice = Some(invoice.clone());
                                    }
                                    ui.add_space(4.0);
                                    let dl_btn = egui::Button::new(
                                        RichText::new("⬇").color(fg).size(11.0),
                                    ).fill(Color32::from_rgb(68, 68, 68)).min_size(Vec2::new(26.0, 22.0));
                                    if ui.add(dl_btn).clicked() {
                                        state.billing_state.download_message =
                                            Some(format!("Downloading {}...", invoice.id));
                                    }
                                });
                            });
                        });

                    if i < invoices.len() - 1 {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().hline(
                            egui::Rangef::new(rect.left(), rect.right()),
                            rect.top(),
                            Stroke::new(1.0, border),
                        );
                    }
                }
            }
        });

    // Download toast
    if let Some(msg) = state.billing_state.download_message.clone() {
        ui.add_space(12.0);
        Frame::none()
            .fill(Color32::from_rgb(30, 55, 30))
            .stroke(Stroke::new(1.0, chart_2))
            .inner_margin(Margin::symmetric(16.0, 10.0))
            .rounding(Rounding::same(8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("[OK] {}", msg)).color(chart_2).size(13.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("[X]").clicked() {
                            state.billing_state.download_message = None;
                        }
                    });
                });
            });
    }

    // Invoice detail modal
    if let Some(invoice) = state.billing_state.selected_invoice.clone() {
        ui.add_space(16.0);
        Frame::none()
            .fill(Color32::from_rgb(45, 45, 58))
            .stroke(Stroke::new(1.5, Color32::from_rgb(124, 58, 202)))
            .inner_margin(Margin::same(20.0))
            .rounding(Rounding::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Invoice {}", invoice.id)).color(fg).size(16.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("[X]").clicked() {
                            state.billing_state.selected_invoice = None;
                        }
                    });
                });
                ui.add_space(12.0);
                detail_row(ui, "Date", &invoice.date, fg, muted);
                detail_row(ui, "Amount", &format!("${:.2} {}", invoice.amount, invoice.currency), fg, muted);
                detail_row(ui, "Status", invoice.status.label(), invoice.status.color(), muted);
                ui.add_space(12.0);
                let dl = egui::Button::new(RichText::new("⬇ Download PDF").color(primary_fg).size(13.0))
                    .fill(primary)
                    .min_size(Vec2::new(140.0, 34.0));
                if ui.add(dl).clicked() {
                    state.billing_state.download_message = Some(format!("Downloading {}...", invoice.id));
                    state.billing_state.selected_invoice = None;
                }
            });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  HELPERS
// ─────────────────────────────────────────────────────────────────────────────

fn section_header(ui: &mut egui::Ui, title: &str, subtitle: &str, fg: Color32, muted: Color32) {
    ui.label(RichText::new(title).color(fg).size(22.0).strong());
    ui.add_space(4.0);
    ui.label(RichText::new(subtitle).color(muted).size(13.0));
}

fn sidebar_item(ui: &mut egui::Ui, label: &str, active: bool, fg: Color32, primary: Color32) -> bool {
    let color = if active { primary } else { fg };
    let btn = egui::Button::new(RichText::new(label).color(color).size(14.0))
        .fill(Color32::TRANSPARENT)
        .min_size(egui::vec2(156.0, 32.0));
    ui.add(btn).clicked()
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str, value_color: Color32, muted: Color32) {
    ui.horizontal(|ui| {
        ui.set_min_height(24.0);
        ui.label(RichText::new(format!("{}: ", label)).color(muted).size(13.0));
        ui.label(RichText::new(value).color(value_color).size(13.0).strong());
    });
}