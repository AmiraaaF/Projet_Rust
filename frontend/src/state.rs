use shared::models::{Project, Task, UserPublic};
use crate::themes::DarkTheme;
use crate::screens::screenBilling::{Plan, BillingInvoice, InvoiceStatus};
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

// ─────────────────────────────────────────────────────────────────────────────
//  BILLING STATE
// ─────────────────────────────────────────────────────────────────────────────

pub struct BillingState {
    pub current_plan: Plan,
    pub pending_plan: Option<Plan>,
    pub show_upgrade_confirm: bool,
    pub show_cancel_confirm: bool,
    pub selected_invoice: Option<BillingInvoice>,
    pub download_message: Option<String>,
    pub invoices: Vec<BillingInvoice>,   
    pub invoices_loaded: bool,           
    pub last_error: Option<String>,      
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
            invoices: Vec::new(),
            invoices_loaded: false,
            last_error: None,
        }
    }
}

impl BillingState {
    pub fn from_plan_name(plan_name: &str) -> Self {
        Self {
            current_plan: Plan::from_str(plan_name),
            ..Default::default()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  APP STATE
// ─────────────────────────────────────────────────────────────────────────────

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
    pub api_client: ApiClient,
    pub is_loading: bool,
}

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

    // ─── BILLING METHODS ───────────────────────────────────────────────────────

    
    pub fn load_subscription_for_user_sync(&mut self, user_id: &str) {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };

        match self.api_client.get_subscription_sync(user_id, &token) {
            Ok(response) => {
                let plan_name = response
                    .get("plan")
                    .and_then(|v| v.as_str())
                    .unwrap_or("free");
                self.billing_state = BillingState::from_plan_name(plan_name);
                eprintln!("✅ Subscription loaded: plan={}", plan_name);
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load subscription: {}", e);
                self.billing_state = BillingState::default();
            }
        }
    }

    pub fn update_plan_sync(&mut self, new_plan: &Plan) -> Result<String, String> {
        let user_id = match &self.current_user {
            Some(u) => u.id.to_string(),
            None => return Err("Non connecté".to_string()),
        };
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return Err("Token manquant".to_string()),
        };

        let plan_name = new_plan.api_name();

        match self.api_client.update_subscription_sync(&user_id, plan_name, &token) {
            Ok(response) => {
                let confirmed_plan = response
                    .get("plan")
                    .and_then(|v| v.as_str())
                    .unwrap_or(plan_name);
                self.billing_state.current_plan = Plan::from_str(confirmed_plan);
                self.billing_state.invoices_loaded = false; 
                eprintln!("✅ Plan updated to: {}", confirmed_plan);
                Ok(confirmed_plan.to_string())
            }
            Err(e) => {
                eprintln!("❌ Failed to update plan: {}", e);
                Err(e)
            }
        }
    }

    
    pub fn cancel_subscription_sync(&mut self) -> Result<(), String> {
        let user_id = match &self.current_user {
            Some(u) => u.id.to_string(),
            None => return Err("Non connecté".to_string()),
        };
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return Err("Token manquant".to_string()),
        };

        match self.api_client.cancel_subscription_sync(&user_id, &token) {
            Ok(_) => {
                self.load_subscription_for_user_sync(&user_id);
                eprintln!("✅ Subscription cancelled");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to cancel: {}", e);
                Err(e)
            }
        }
    }

    
    pub fn load_invoices_sync(&mut self) {
        if self.billing_state.invoices_loaded {
            return; 
        }

        let user_id = match &self.current_user {
            Some(u) => u.id.to_string(),
            None => return,
        };
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };

        match self.api_client.get_invoices_sync(&user_id, &token) {
            Ok(response) => {
                let invoices = parse_invoices_from_response(&response);
                self.billing_state.invoices = invoices;
                self.billing_state.invoices_loaded = true;
                eprintln!("✅ Loaded {} invoices", self.billing_state.invoices.len());
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load invoices: {}", e);
                self.billing_state.invoices = Vec::new();
                self.billing_state.invoices_loaded = true; 
            }
        }
    }
}


fn parse_invoices_from_response(response: &serde_json::Value) -> Vec<BillingInvoice> {
    let data = match response.get("data").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    data.iter()
        .filter_map(|item| {
            let id = item.get("id")?.as_str()?.to_string();
            let amount = item.get("amount")?.as_f64()?;
            let currency = item
                .get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("USD")
                .to_string();
            let status_str = item
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("issued");
            let issued_at = item
                .get("issued_at")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let status = match status_str {
                "paid" => InvoiceStatus::Paid,
                "overdue" => InvoiceStatus::Overdue,
                _ => InvoiceStatus::Pending,
            };

            
            let date = if issued_at.len() >= 10 {
                issued_at[..10].to_string()
            } else {
                issued_at.clone()
            };

            let plan = item
                .get("plan")
                .and_then(|v| v.as_str())
                .unwrap_or("—")
                .to_string();

            Some(BillingInvoice {
                id: format!("INV-{}", &id[..8].to_uppercase()),
                date,
                plan,
                amount,
                currency,
                status,
            })
        })
        .collect()
}