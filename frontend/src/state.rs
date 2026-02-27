use shared::models::{Project, Task, UserPublic};
use crate::themes::DarkTheme;

#[derive(Clone, Debug)]
pub enum Screen {
    Login,
    Register,
    Dashboard,
    Projects,
    ProjectDetail,
}

pub struct AppState {
    pub current_screen: Screen,
    pub current_user: Option<UserPublic>,
    pub token: Option<String>,
    pub email_input: String,
    pub password_input: String,
    pub name_input: String,
    pub project_name_input: String,
    pub project_description_input: String,
    pub task_title_input: String,
    pub projects: Vec<Project>,
    pub current_project: Option<Project>,
    pub current_tasks: Vec<Task>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub api_url: String,
    pub theme: DarkTheme,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::Login,
            current_user: None,
            token: None,
            email_input: String::new(),
            password_input: String::new(),
            name_input: String::new(),
            project_name_input: String::new(),
            project_description_input: String::new(),
            task_title_input: String::new(),
            projects: Vec::new(),
            current_project: None,
            current_tasks: Vec::new(),
            error_message: None,
            success_message: None,
            api_url: "http://localhost".to_string(),
            theme: DarkTheme::new(),
        }
    }


    pub fn go_to(&mut self, screen: Screen) {
        self.current_screen = screen;
        self.error_message = None;
        self.success_message = None;
    }

    pub fn clear_forms(&mut self) {
        self.email_input.clear();
        self.password_input.clear();
        self.name_input.clear();
        self.project_name_input.clear();
        self.project_description_input.clear();
        self.task_title_input.clear();
    }

    pub fn logout(&mut self) {
        self.current_user = None;
        self.token = None;
        self.error_message = None;
        self.success_message = None;
        self.clear_forms();
        self.projects.clear();
        self.current_project = None;
        self.current_tasks.clear();
        self.current_screen = Screen::Login;
    }
}