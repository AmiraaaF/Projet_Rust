mod api;
mod screens;
mod state;

use eframe::egui;
use screens::{screenAuth, screenDashboard, screenProject};
use state::{AppState, Screen};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mini-SaaS Dashboard",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals {
                dark_mode: true,
                panel_fill: egui::Color32::from_rgb(20, 20, 35),
                window_fill: egui::Color32::from_rgb(30, 30, 50),
                extreme_bg_color: egui::Color32::from_rgb(10, 10, 20),
                override_text_color: Some(egui::Color32::from_rgb(220, 220, 255)),
                selection: egui::style::Selection {
                    bg_fill: egui::Color32::from_rgb(80, 60, 200),
                    stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
                },
                ..egui::Visuals::dark()
            });
            Box::<MyApp>::default()
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