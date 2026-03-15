use shared::models::{Project, Task, UserPublic};
use crate::themes::DarkTheme;
use crate::screens::screenBilling::{Plan, BillingInvoice, InvoiceStatus};
use crate::screens::screenNotifications::{Notification, NotifStatus, FilterTab};
use crate::api::ApiClient;
use uuid;

#[derive(Clone, Debug)]
pub enum Screen {
    Login,
    Register,
    Dashboard,
    Projects,
    ProjectDetail,
    Billing,
    Notifications,
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
//  NOTIFICATION STATE
// ─────────────────────────────────────────────────────────────────────────────

pub struct NotificationState {
    pub notifications: Vec<Notification>,
    pub unread_count:  u64,
    pub loaded:        bool,
    pub active_filter: FilterTab,
    pub toast_message: Option<String>,
    pub toast_time: std::time::Instant,
    pub last_poll: std::time::Instant,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            unread_count:  0,
            loaded:        false,
            active_filter: FilterTab::All,
            toast_message: None,
            toast_time: std::time::Instant::now(),
            last_poll: std::time::Instant::now(),
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
    // Champs pour la gestion des tâches
    pub new_task_title_input: String,
    pub new_task_description_input: String,
    pub show_add_task_form: bool,
    pub dragging_task_id: Option<uuid::Uuid>,
    // Données
    pub projects: Vec<Project>,
    pub current_project: Option<Project>,
    pub current_tasks: Vec<Task>,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub api_url: String,
    pub theme: DarkTheme,
    pub billing_state: BillingState,
    pub notif_state: NotificationState,

    pub api_client: ApiClient,
    pub is_loading: bool,
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}

impl AppState {
    pub fn new() -> Self {
        let api_url = "http://localhost:3000".to_string();
        Self {
            current_screen:            Screen::Login,
            current_user:              None,
            token:                     None,
            email_input:               String::new(),
            password_input:            String::new(),
            name_input:                String::new(),
            project_name_input:        String::new(),
            project_description_input: String::new(),
            task_title_input: String::new(),
            // Nouveaux champs pour les tâches
            new_task_title_input: String::new(),
            new_task_description_input: String::new(),
            show_add_task_form: false,
            dragging_task_id: None,
            // Données
            projects: Vec::new(),
            current_project: None,
            current_tasks: Vec::new(),
            error_message: None,
            success_message: None,
            api_url: api_url.clone(),
            theme: DarkTheme::new(),
            billing_state: BillingState::default(),
            notif_state: NotificationState::default(),
            api_client: ApiClient::new(api_url),
            is_loading: false,
        }
    }

    pub fn go_to(&mut self, screen: Screen) {
        self.current_screen = screen;
        self.error_message  = None;
        self.success_message = None;
    }

    pub fn clear_forms(&mut self) {
        self.email_input.clear();
        self.password_input.clear();
        self.name_input.clear();
        self.project_name_input.clear();
        self.project_description_input.clear();
        self.task_title_input.clear();
        self.new_task_title_input.clear();
        self.new_task_description_input.clear();
        self.show_add_task_form = false;
        self.dragging_task_id = None;
    }

    pub fn logout(&mut self) {
        self.current_user    = None;
        self.token           = None;
        self.error_message   = None;
        self.success_message = None;
        self.clear_forms();
        // self.projects.clear(); 
        self.current_project = None;
        self.current_tasks.clear();
        self.billing_state = BillingState::default();
        self.notif_state   = NotificationState::default();
        self.current_screen = Screen::Login;
    }

    // ─── PROJECT METHODS ───────────────────────────────────────────────────────
    
    pub fn load_projects_sync(&mut self) {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };

        self.is_loading = true;
        match self.api_client.get_projects_sync(1, 50, &token) {
            Ok(response) => {
                self.projects = response.data;
                self.error_message = None;
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load projects: {}", e);
                self.error_message = Some(format!("Failed to load projects: {}", e));
            }
        }
        self.is_loading = false;
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
                    .get("plan").and_then(|v| v.as_str()).unwrap_or("free");
                self.billing_state = BillingState::from_plan_name(plan_name);
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load subscription: {}", e);
                self.billing_state = BillingState::default();
            }
        }
    }

    pub fn update_plan_sync(&mut self, new_plan: &Plan) -> Result<String, String> {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return Err("Not logged in".to_string()) };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return Err("Missing token".to_string()) };
        let plan_name = new_plan.api_name();

        match self.api_client.update_subscription_sync(&user_id, plan_name, &token) {
            Ok(response) => {
                let confirmed = response.get("plan").and_then(|v| v.as_str()).unwrap_or(plan_name);
                self.billing_state.current_plan  = Plan::from_str(confirmed);
                self.billing_state.invoices_loaded = false;
                // Trigger notification
                if let Some(token) = &self.token {
                    self.api_client.send_event_sync(&user_id, "plan_upgraded", serde_json::json!({"plan": confirmed}), token).ok();
                }
                Ok(confirmed.to_string())
            }
            Err(e) => Err(e),
        }
    }

    pub fn cancel_subscription_sync(&mut self) -> Result<(), String> {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return Err("Not logged in".to_string()) };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return Err("Missing token".to_string()) };

        match self.api_client.cancel_subscription_sync(&user_id, &token) {
            Ok(_) => {
                self.load_subscription_for_user_sync(&user_id);
                // Trigger notification
                if let Some(token) = &self.token {
                    self.api_client.send_event_sync(&user_id, "plan_cancelled", serde_json::json!({}), token).ok();
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn load_invoices_sync(&mut self) {
        if self.billing_state.invoices_loaded { return; }
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };

        match self.api_client.get_invoices_sync(&user_id, &token) {
            Ok(response) => {
                self.billing_state.invoices        = parse_invoices_from_response(&response);
                self.billing_state.invoices_loaded = true;
            }
            Err(_) => {
                self.billing_state.invoices        = Vec::new();
                self.billing_state.invoices_loaded = true;
            }
        }
    }

    // ─── NOTIFICATION METHODS ──────────────────────────────────────────────────


    pub fn load_notifications_sync(&mut self) {
        if self.notif_state.loaded { return; }
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };

        match self.api_client.get_notifications_sync(&user_id, &token) {
            Ok(resp) => {
                self.notif_state.notifications = parse_notifications(&resp);
                self.notif_state.unread_count  =
                    resp.get("unread_count").and_then(|v| v.as_u64()).unwrap_or(0);
                self.notif_state.loaded = true;
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load notifications: {}", e);
                self.notif_state.loaded = true; 
            }
        }
    }

    
    pub fn invalidate_notifications(&mut self) {
        self.notif_state.loaded = false;
    }

    
    pub fn poll_notifications_sync(&mut self) {
        if self.current_user.is_none() || self.token.is_none() {
            return;
        }
        if self.notif_state.last_poll.elapsed().as_secs_f32() < 1.0 {
            return;
        }
        self.notif_state.last_poll = std::time::Instant::now();

        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };

        match self.api_client.get_notifications_sync(&user_id, &token) {
            Ok(resp) => {
                self.notif_state.notifications = parse_notifications(&resp);
                self.notif_state.unread_count  =
                    resp.get("unread_count").and_then(|v| v.as_u64()).unwrap_or(0);
                self.notif_state.loaded = true;
            }
            Err(_) => {
                
            }
        }
    }

    pub fn mark_notification_read_sync(&mut self, notif_id: &str) {
        let token = match &self.token { Some(t) => t.clone(), None => return };
        match self.api_client.mark_notif_read_sync(notif_id, &token) {
            Ok(_) => {
                if let Some(n) = self.notif_state.notifications.iter_mut().find(|n| n.id == notif_id) {
                    n.status = NotifStatus::Read;
                }
                self.notif_state.unread_count = self.notif_state.unread_count.saturating_sub(1);
            }
            Err(e) => {
                self.notif_state.toast_message = Some(format!("⚠ {}", e));
                self.notif_state.toast_time = std::time::Instant::now();
            }
        }
    }

    pub fn mark_all_read_sync(&mut self) {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };
        match self.api_client.mark_all_read_sync(&user_id, &token) {
            Ok(_) => {
                for n in self.notif_state.notifications.iter_mut() { n.status = NotifStatus::Read; }
                self.notif_state.unread_count  = 0;
                self.notif_state.toast_message = Some("✅ All notifications marked as read".to_string());
                self.notif_state.toast_time = std::time::Instant::now();
            }
            Err(e) => {
                self.notif_state.toast_message = Some(format!("⚠ {}", e));
                self.notif_state.toast_time = std::time::Instant::now();
            }
        }
    }

    pub fn delete_notification_sync(&mut self, notif_id: &str) {
        let token = match &self.token { Some(t) => t.clone(), None => return };
        match self.api_client.delete_notif_sync(notif_id, &token) {
            Ok(_) => {
                let was_unread = self.notif_state.notifications.iter()
                    .find(|n| n.id == notif_id)
                    .map(|n| n.status == NotifStatus::Sent)
                    .unwrap_or(false);
                self.notif_state.notifications.retain(|n| n.id != notif_id);
                if was_unread {
                    self.notif_state.unread_count = self.notif_state.unread_count.saturating_sub(1);
                }
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load invoices: {}", e);
                self.billing_state.invoices = Vec::new();
                self.billing_state.invoices_loaded = true; 
            }
        }
    }

    pub fn clear_read_notifications_sync(&mut self) {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };

        match self.api_client.clear_read_sync(&user_id, &token) {
            Ok(_) => {
                self.notif_state.notifications.retain(|n| n.status != crate::screens::screenNotifications::NotifStatus::Read);
                self.notif_state.unread_count = self.notif_state.notifications.iter().filter(|n| n.status != crate::screens::screenNotifications::NotifStatus::Read).count() as u64;
                self.notif_state.toast_message = Some("✅ Read notifications cleared".to_string());
                self.notif_state.toast_time = std::time::Instant::now();
            }
            Err(e) => {
                self.notif_state.toast_message = Some(format!("⚠ {}", e));
                self.notif_state.toast_time = std::time::Instant::now();
            }
        }
    }

    // ─── PROJECT METHODS ───────────────────────────────────────────────────────

    pub fn create_project_sync(&mut self, name: &str, description: Option<&str>) -> Result<(), String> {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return Err("Non connecté".to_string()),
        };

        match self.api_client.create_project_sync(name, description, &token) {
            Ok(project) => {
                self.projects.push(project.clone());
                eprintln!("✅ Project created: {}", project.name);
                // Trigger notification
                if let Some(user) = &self.current_user {
                    let _ = self.api_client.send_event_sync(&user.id.to_string(), "project_created", serde_json::json!({"name": name}), self.token.as_deref().unwrap_or(""));
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to create project: {}", e);
                Err(e)
            }
        }
    }

    // ─── TASK METHODS ──────────────────────────────────────────────────────────

    pub fn create_task_sync(&mut self, project_id: &str, title: &str, description: Option<&str>) -> Result<(), String> {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return Err("Non connecté".to_string()),
        };

        match self.api_client.create_task_sync(project_id, title, description, &token) {
            Ok(task) => {
                self.current_tasks.push(task.clone());
                eprintln!("✅ Task created: {}", task.title);
                // Trigger notification
                if let Some(user) = &self.current_user {
                    let _ = self.api_client.send_event_sync(&user.id.to_string(), "task_assigned", serde_json::json!({"title": title, "project": project_id}), self.token.as_deref().unwrap_or(""));
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to create task: {}", e);
                Err(e)
            }
        }
    }

    pub fn load_tasks_sync(&mut self, project_id: &str) {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return,
        };

        match self.api_client.get_tasks_sync(project_id, 1, 100, &token) {
            Ok(response) => {
                self.current_tasks = response.data;
                eprintln!("✅ Loaded {} tasks", self.current_tasks.len());
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load tasks: {}", e);
            }
        }
    }

    pub fn update_task_status_sync(&mut self, project_id: &str, task_id: &str, status: &str) -> Result<(), String> {
        let token = match &self.token {
            Some(t) => t.clone(),
            None => return Err("Non connecté".to_string()),
        };

        match self.api_client.update_task_status_sync(project_id, task_id, status, &token) {
            Ok(updated_task) => {
                if let Some(task) = self.current_tasks.iter_mut().find(|t| t.id.to_string() == task_id) {
                    task.status = updated_task.status.clone();
                }
                eprintln!("✅ Task status updated: {}", status);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to update task status: {}", e);
                Err(e)
            }
        }
    }
}


fn parse_invoices_from_response(response: &serde_json::Value) -> Vec<BillingInvoice> {
    let data = match response.get("data").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    data.iter().filter_map(|item| {
        let id       = item.get("id")?.as_str()?.to_string();
        let amount   = item.get("amount")?.as_f64()?;
        let currency = item.get("currency").and_then(|v| v.as_str()).unwrap_or("USD").to_string();
        let stat_str = item.get("status").and_then(|v| v.as_str()).unwrap_or("issued");
        let issued   = item.get("issued_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let plan     = item.get("plan").and_then(|v| v.as_str()).unwrap_or("—").to_string();

        let status = match stat_str {
            "paid"    => InvoiceStatus::Paid,
            "overdue" => InvoiceStatus::Overdue,
            _         => InvoiceStatus::Pending,
        };
        let date = if issued.len() >= 10 { issued[..10].to_string() } else { issued };

        Some(BillingInvoice {
            id:       format!("INV-{}", &id[..8].to_uppercase()),
            date,
            plan,
            amount,
            currency,
            status,
        })
    }).collect()
}

fn parse_notifications_value(resp: &serde_json::Value) -> Vec<crate::screens::screenNotifications::Notification> {
    let data = resp.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    data.iter().filter_map(|item| {
        let id = item.get("id")?.as_str()?.to_string();
        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message = item.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let notif_type = item.get("notification_type").and_then(|v| v.as_str()).unwrap_or("inapp");
        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("sent");
        let created_at = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Some(crate::screens::screenNotifications::Notification {
            id,
            title,
            message,
            notif_type: crate::screens::screenNotifications::NotifType::from_str(notif_type),
            status: crate::screens::screenNotifications::NotifStatus::from_str(status),
            created_at,
        })
    }).collect()
}

// Backwards-compatible wrapper used by existing call sites
fn parse_notifications(resp: &serde_json::Value) -> Vec<crate::screens::screenNotifications::Notification> {
    parse_notifications_value(resp)
}

