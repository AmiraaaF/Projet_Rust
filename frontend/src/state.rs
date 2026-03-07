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
    Tasks,   
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
    //tasks
    pub task_title_input: String,
    pub task_description_input: String,
    pub task_priority_input: String,   
    pub task_status_input: String,     
    pub task_assignee_input: String,    
    pub task_deadline_input: String,    
    pub task_project_id_input: String,  
    pub show_task_form: bool,          
    pub tasks_loaded: bool,  
    // Filtres pour les tâches
    pub filter_my_tasks: bool,                  
    pub filter_status: Option<String>,       
    pub filter_changed: bool,                  

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
            task_description_input: String::new(),
            task_priority_input: "medium".to_string(),
            task_status_input: "todo".to_string(),
            task_assignee_input: String::new(),
            task_deadline_input: String::new(),
            task_project_id_input: String::new(),
            show_task_form: false,
            tasks_loaded: false,
            // Initialisation des filtres
            filter_my_tasks: false,
            filter_status: None,
            filter_changed: false,

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
        self.task_description_input.clear();
        self.task_priority_input = "medium".to_string();
        self.task_status_input   = "todo".to_string();
        self.task_assignee_input.clear();
        self.task_deadline_input.clear();
        self.task_project_id_input.clear();
        self.show_task_form = false;
        self.tasks_loaded = false;
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
        self.tasks_loaded = false;
        self.billing_state = BillingState::default();
        self.current_screen = Screen::Login;
    }

    // ─── TASKS METHODS ──────────────────────────────────────────────────────

    // Charge les tâches avec filtres optionnels
    pub fn load_tasks_sync(&mut self) {
        use uuid::Uuid;
        use chrono::DateTime;
        // Recharger seulement si:
        // 1. Les tâches ne sont pas encore chargées, OU
        // 2. Les filtres ont changé
        if self.tasks_loaded && !self.filter_changed { return; }

        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };

        // Construire les paramètres de filtre
        let assignee_id = if self.filter_my_tasks {
            self.current_user.as_ref().map(|u| u.id.to_string())
        } else {
            None
        };
        let status = self.filter_status.clone();

        match self.api_client.list_tasks_sync(
            assignee_id.as_deref(),
            status.as_deref(),
            None,
            &token,
        ) {
            Ok(responses) => {
                self.current_tasks = responses
                    .into_iter()
                    .filter_map(|r| {
                        let id         = Uuid::parse_str(&r.id).ok()?;
                        let project_id = Uuid::parse_str(&r.project_id).ok()?;
                        let assignee_id = r.assignee_id
                            .as_deref()
                            .and_then(|s| Uuid::parse_str(s).ok());
                        let deadline = r.deadline
                            .as_deref()
                            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                            .map(|d| d.with_timezone(&chrono::Utc));
                        let created_at = DateTime::parse_from_rfc3339(&r.created_at)
                            .ok()?.with_timezone(&chrono::Utc);
                        let updated_at = DateTime::parse_from_rfc3339(&r.updated_at)
                            .ok()?.with_timezone(&chrono::Utc);
                        Some(Task {
                            id, project_id, assignee_id,
                            title: r.title,
                            description: r.description,
                            status: r.status,
                            priority: r.priority,
                            deadline, created_at, updated_at,
                            assignee_name: r.assignee_name,
                            project_name: r.project_name,
                        })
                    })
                    .collect();
                self.tasks_loaded = true;
                self.filter_changed = false;  // Réinitialiser le flag après chargement
                eprintln!("{} tâche(s) chargée(s)", self.current_tasks.len());
            }
            Err(e) => eprintln!("Impossible de charger les tâches: {}", e),
        }
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