mod api;
mod screens;
mod state;
mod themes;

use eframe::egui;
use screens::{screenAuth, screenDashboard, screenProject};
use state::{AppState, Screen};
use themes::DarkTheme;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mini-SaaS Dashboard",
        options,
        Box::new(|cc| {
            let theme = DarkTheme::new();
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = theme.card;
            visuals.window_fill = theme.background;
            visuals.extreme_bg_color = theme.background;
            visuals.faint_bg_color = theme.card;
            visuals.override_text_color = Some(theme.foreground);
            visuals.selection.bg_fill = theme.primary;
            visuals.selection.stroke = egui::Stroke::new(1.0, theme.primary_foreground);
            visuals.hyperlink_color = theme.primary;
            visuals.error_fg_color = theme.destructive;
            visuals.warn_fg_color = theme.accent;
            visuals.code_bg_color = theme.muted;
            cc.egui_ctx.set_visuals(visuals);
            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    state: AppState,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.state.current_screen {
            Screen::Login => screenAuth::login_screen(ctx, &mut self.state),
            Screen::Register => screenAuth::register_screen(ctx, &mut self.state),
            Screen::Dashboard => screenDashboard::dashboard_screen(ctx, &mut self.state),
            Screen::Projects => screenProject::projects_screen(ctx, &mut self.state),
            Screen::ProjectDetail => screenProject::project_detail_screen(ctx, &mut self.state),
        }
    }
}