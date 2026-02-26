mod api;
mod ui;
mod state;

use eframe::egui;
use state::{AppState, Screen};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mini-SaaS Dashboard",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
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
            Screen::Login => ui::login_screen(ctx, &mut self.state),
            Screen::Register => ui::register_screen(ctx, &mut self.state),
            Screen::Dashboard => ui::dashboard_screen(ctx, &mut self.state),
            Screen::Projects => ui::projects_screen(ctx, &mut self.state),
            Screen::ProjectDetail => ui::project_detail_screen(ctx, &mut self.state),
        }
    }
}