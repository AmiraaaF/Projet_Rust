use shared::models::{Project, Task, UserPublic};
use crate::themes::DarkTheme;
use crate::screens::screenBilling::{Plan, Invoice};
use crate::api::ApiClient;

#[derive(Clone, Debug)]
pub enum Screen {
    Login,
    Register,
    Dashboard,
    Projects,
    ProjectDetail,
    Billing,
}



pub struct BillingState {
    pub current_plan: Plan,
    pub pending_plan: Option<Plan>,
    pub show_upgrade_confirm: bool,
    pub show_cancel_confirm: bool,
    pub selected_invoice: Option<Invoice>,
    pub download_message: Option<String>,
    pub plan_changed: bool, // Flag for API sync
}

impl Default for BillingState {
    fn default() -> Self {
        Self {
            current_plan: Plan::Free,          
            pending_plan: None,
            show_upgrade_confirm: false,
            show_cancel_confirm: false,
            selected_invoice: None,
            download_message: None,
            plan_changed: false,
        }
    }
}

impl BillingState {
    /// Create BillingState from DB plan name
    pub fn from_db(plan_name: &str) -> Self {
        let current_plan = match plan_name {
            "pro" => crate::screens::screenBilling::Plan::Pro,
            "enterprise" => crate::screens::screenBilling::Plan::Enterprise,
            _ => crate::screens::screenBilling::Plan::Free,
        };

        Self {
            current_plan,
            pending_plan: None,
            show_upgrade_confirm: false,
            show_cancel_confirm: false,
            selected_invoice: None,
            download_message: None,
            plan_changed: false,
        }
    }
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
    pub billing_state: BillingState,
    pub api_client: crate::api::ApiClient,    pub is_loading: bool, // Track async operations
    pub pending_login: bool,
    pub pending_register: bool,}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let api_url = "http://localhost:3001".to_string();
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
            api_url: api_url.clone(),
            theme: DarkTheme::new(),
            billing_state: BillingState::default(),
            api_client: ApiClient::new(api_url),
            is_loading: false,
            pending_login: false,
            pending_register: false,
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
        self.billing_state = BillingState::default();
        self.current_screen = Screen::Login;
    }

    /// Load subscription for current user from API
    pub fn load_subscription_for_user_sync(&mut self, user_id: &str) {
        if let Some(token) = &self.token {
            match self.api_client.get_subscription_sync(user_id, token) {
                Ok(response) => {
                    if let Some(plan_name) = response.get("plan").and_then(|v| v.as_str()) {
                        self.billing_state = BillingState::from_db(plan_name);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load subscription: {}", e);
                    self.billing_state = BillingState::default();
                }
            }
        }
    }

    /// Attempt login - returns (success, error_message)
    pub fn attempt_login(&mut self, email: &str, password: &str) -> (bool, String) {
        // This would need to be async but we're in sync context
        // For now, return an error directing to setup the server
        (false, "⚠️ Serveur de connexion non disponible. Assurez-vous que les services backend tournent sur http://localhost:3001".to_string())
    }

    /// Attempt register - returns (success, error_message)
    pub fn attempt_register(&mut self, email: &str, name: &str, password: &str) -> (bool, String) {
        (false, "⚠️ Serveur d'enregistrement non disponible. Assurez-vous que les services backend tournent sur http://localhost:3001".to_string())
    }
}