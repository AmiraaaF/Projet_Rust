use chrono::Datelike;
use shared::models::{Project, Task, UserPublic};
use crate::themes::DarkTheme;
use crate::screens::screenBilling::{Plan, BillingInvoice, InvoiceStatus};
use crate::screens::screenNotifications::{Notification, NotifType, NotifStatus, FilterTab};
use crate::api::ApiClient;

#[derive(Clone, Debug)]
pub enum Screen {
    Login,
    Register,
    Dashboard,
    Projects,
    ProjectDetail,
    Billing,
    Notifications,
    Profile,
    Todo,
    Calendar,
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
//  TODO STATE
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TodoStatus {
    Todo,
    InProgress,
    Done,
}

impl TodoStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "in_progress" => Self::InProgress,
            "done"        => Self::Done,
            _             => Self::Todo,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            Self::Todo       => "To Do",
            Self::InProgress => "In Progress",
            Self::Done       => "Done",
        }
    }
    pub fn icon(&self) -> &str {
        match self {
            Self::Todo       => "⬜",
            Self::InProgress => "🔄",
            Self::Done       => "✅",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodoPriority {
    Low,
    Medium,
    High,
}

impl TodoPriority {
    pub fn from_str(s: &str) -> Self {
        match s {
            "high" => Self::High,
            "low"  => Self::Low,
            _      => Self::Medium,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            Self::High   => "High",
            Self::Medium => "Medium",
            Self::Low    => "Low",
        }
    }
    pub fn emoji(&self) -> &str {
        match self {
            Self::High   => "🔴",
            Self::Medium => "🟡",
            Self::Low    => "🟢",
        }
    }
    pub fn next(&self) -> Self {
        match self {
            Self::Low    => Self::Medium,
            Self::Medium => Self::High,
            Self::High   => Self::Low,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TodoItem {
    pub id:           String,
    pub title:        String,
    pub description:  String,
    pub status:       TodoStatus,
    pub priority:     TodoPriority,
    pub deadline:     Option<String>,       // "YYYY-MM-DD"
    pub project_name: Option<String>,
    pub created_at:   String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TodoFilter {
    All,
    Active,
    InProgress,
    Done,
    HighPriority,
    HasDeadline,
}

impl TodoFilter {
    pub fn label(&self) -> &str {
        match self {
            Self::All         => "All",
            Self::Active      => "To Do",
            Self::InProgress  => "In Progress",
            Self::Done        => "Done",
            Self::HighPriority => "🔴 High",
            Self::HasDeadline  => "📅 Deadline",
        }
    }
}

pub struct TodoState {
    pub items:            Vec<TodoItem>,
    pub filter:           TodoFilter,
    pub show_form:        bool,
    pub form_title:       String,
    pub form_description: String,
    pub form_priority:    TodoPriority,
    pub form_deadline:    String,
    pub form_project:     String,
    pub editing_id:       Option<String>,
    pub form_error:       Option<String>,
    pub next_id:          u64,
    pub confirm_delete_id: Option<String>,
}

impl Default for TodoState {
    fn default() -> Self {
        Self {
            items:            Vec::new(),
            filter:           TodoFilter::All,
            show_form:        false,
            form_title:       String::new(),
            form_description: String::new(),
            form_priority:    TodoPriority::Medium,
            form_deadline:    String::new(),
            form_project:     String::new(),
            editing_id:       None,
            form_error:       None,
            next_id:          1,
            confirm_delete_id: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PROFILE STATE
// ─────────────────────────────────────────────────────────────────────────────

pub struct ProfileState {
    pub is_editing:  bool,
    pub edit_name:   String,
    pub edit_email:  String,
    pub success_msg: Option<String>,
    pub error_msg:   Option<String>,
    pub msg_time:    std::time::Instant,
}

impl Default for ProfileState {
    fn default() -> Self {
        Self {
            is_editing:  false,
            edit_name:   String::new(),
            edit_email:  String::new(),
            success_msg: None,
            error_msg:   None,
            msg_time:    std::time::Instant::now(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  CALENDAR STATE
// ─────────────────────────────────────────────────────────────────────────────

pub struct CalendarState {
    pub year:         i32,
    pub month:        u32,
    pub selected_day: Option<u32>,
}

impl Default for CalendarState {
    fn default() -> Self {
        let now = chrono::Local::now();
        Self {
            year:         now.year(),
            month:        now.month(),
            selected_day: None,
        }
    }
}

impl CalendarState {
    pub fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
        self.selected_day = None;
    }

    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
        self.selected_day = None;
    }

    pub fn month_name(&self) -> &str {
        match self.month {
            1  => "January",  2  => "February", 3  => "March",
            4  => "April",    5  => "May",       6  => "June",
            7  => "July",     8  => "August",    9  => "September",
            10 => "October",  11 => "November",  12 => "December",
            _  => "?",
        }
    }

    /// Returns 0=Monday … 6=Sunday for the 1st of this month
    pub fn first_weekday(&self) -> u32 {
        use chrono::NaiveDate;
        if let Some(d) = NaiveDate::from_ymd_opt(self.year, self.month, 1) {
            use chrono::Datelike;
            d.weekday().num_days_from_monday()
        } else { 0 }
    }

    pub fn days_in_month(&self) -> u32 {
        use chrono::NaiveDate;
        let (ny, nm) = if self.month == 12 { (self.year + 1, 1) } else { (self.year, self.month + 1) };
        if let (Some(next), Some(cur)) = (
            NaiveDate::from_ymd_opt(ny, nm, 1),
            NaiveDate::from_ymd_opt(self.year, self.month, 1),
        ) {
            (next - cur).num_days() as u32
        } else { 30 }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  APP STATE
// ─────────────────────────────────────────────────────────────────────────────

pub struct AppState {
    pub current_screen:              Screen,
    pub current_user:                Option<UserPublic>,
    pub token:                       Option<String>,
    pub email_input:                 String,
    pub password_input:              String,
    pub name_input:                  String,
    pub project_name_input:          String,
    pub project_description_input:   String,
    pub task_title_input:            String,
    pub projects:                    Vec<Project>,
    pub current_project:             Option<Project>,
    pub current_tasks:               Vec<Task>,
    pub error_message:               Option<String>,
    pub success_message:             Option<String>,
    pub api_url:                     String,
    pub theme:                       DarkTheme,
    pub billing_state:               BillingState,
    pub notif_state:                 NotificationState,
    pub todo_state:                  TodoState,
    pub profile_state:               ProfileState,
    pub calendar_state:              CalendarState,
    pub api_client:                  ApiClient,
    pub is_loading:                  bool,
}

impl Default for AppState {
    fn default() -> Self { Self::new() }
}

impl AppState {
    pub fn new() -> Self {
        let api_url = "http://localhost:3001".to_string();
        Self {
            current_screen:            Screen::Login,
            current_user:              None,
            token:                     None,
            email_input:               String::new(),
            password_input:            String::new(),
            name_input:                String::new(),
            project_name_input:        String::new(),
            project_description_input: String::new(),
            task_title_input:          String::new(),
            projects:                  Vec::new(),
            current_project:           None,
            current_tasks:             Vec::new(),
            error_message:             None,
            success_message:           None,
            api_url:                   api_url.clone(),
            theme:                     DarkTheme::new(),
            billing_state:             BillingState::default(),
            notif_state:               NotificationState::default(),
            todo_state:                TodoState::default(),
            profile_state:             ProfileState::default(),
            calendar_state:            CalendarState::default(),
            api_client:                ApiClient::new(api_url),
            is_loading:                false,
        }
    }

    pub fn go_to(&mut self, screen: Screen) {
        self.current_screen  = screen;
        self.error_message   = None;
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
        self.current_user    = None;
        self.token           = None;
        self.error_message   = None;
        self.success_message = None;
        self.clear_forms();
        self.projects.clear();
        self.current_project = None;
        self.current_tasks.clear();
        self.billing_state   = BillingState::default();
        self.notif_state     = NotificationState::default();
        self.todo_state      = TodoState::default();
        self.profile_state   = ProfileState::default();
        self.calendar_state  = CalendarState::default();
        self.current_screen  = Screen::Login;
    }

    // ─── BILLING METHODS ───────────────────────────────────────────────────────

    pub fn load_subscription_for_user_sync(&mut self, user_id: &str) {
        let token = match &self.token { Some(t) => t.clone(), None => return };
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
                self.send_event_sync("plan_upgraded", serde_json::json!({"plan": confirmed}));
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
                self.send_event_sync("plan_cancelled", serde_json::json!({}));
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
        if self.current_user.is_none() || self.token.is_none() { return; }
        if self.notif_state.last_poll.elapsed().as_secs_f32() < 1.0 { return; }
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
            Err(_) => {}
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
                self.notif_state.toast_message = Some(format!("⚠ {}", e));
                self.notif_state.toast_time = std::time::Instant::now();
            }
        }
    }

    pub fn clear_read_notifications_sync(&mut self) {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };
        match self.api_client.clear_read_sync(&user_id, &token) {
            Ok(_) => {
                self.notif_state.notifications.retain(|n| n.status != NotifStatus::Read);
                self.notif_state.toast_message = Some("✅ Read notifications cleared".to_string());
                self.notif_state.toast_time = std::time::Instant::now();
            }
            Err(e) => {
                self.notif_state.toast_message = Some(format!("⚠ {}", e));
                self.notif_state.toast_time = std::time::Instant::now();
            }
        }
    }

    pub fn send_event_sync(&mut self, event_type: &str, payload: serde_json::Value) {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };

        match self.api_client.send_notif_event_sync(&user_id, event_type, payload, &token) {
            Ok(_) => { self.notif_state.loaded = false; }
            Err(e) => eprintln!("⚠️ Failed to send event: {}", e),
        }
    }

    // ─── TODO METHODS (PERSONAL TASKS) ────────────────────────────────────────

    pub fn load_personal_tasks_sync(&mut self) {
        let token = match &self.token { Some(t) => t.clone(), None => return };

        match self.api_client.list_personal_tasks_sync(&token) {
            Ok(tasks) => {
                self.todo_state.items = tasks.into_iter().map(|task| {
                    let deadline_str = task.deadline.map(|dt| dt.format("%Y-%m-%d").to_string());
                    TodoItem {
                        id:           task.id.to_string(),
                        title:        task.title,
                        description:  task.description.unwrap_or_default(),
                        status:       TodoStatus::from_str(&task.status),
                        priority:     TodoPriority::from_str(&task.priority),
                        deadline:     deadline_str,
                        project_name: None,
                        created_at:   task.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    }
                }).collect();
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load personal tasks: {}", e);
                self.todo_state.items.clear();
            }
        }
    }

    pub fn add_todo_sync(&mut self) -> bool {
        let title = self.todo_state.form_title.trim().to_string();
        if title.is_empty() {
            self.todo_state.form_error = Some("Title cannot be empty".to_string());
            return false;
        }

        let token = match &self.token { Some(t) => t.clone(), None => return false };

        let deadline_str = if self.todo_state.form_deadline.trim().is_empty() {
            None
        } else {
            Some(self.todo_state.form_deadline.trim())
        };

        let priority_str = match self.todo_state.form_priority {
            TodoPriority::Low    => "low",
            TodoPriority::Medium => "medium",
            TodoPriority::High   => "high",
        };

        let description = if self.todo_state.form_description.trim().is_empty() {
            None
        } else {
            Some(self.todo_state.form_description.trim())
        };

        match self.api_client.create_personal_task_sync(
            &title,
            description,
            Some(priority_str),
            deadline_str,
            &token,
        ) {
            Ok(task) => {
                let deadline_display = task.deadline.map(|dt| dt.format("%Y-%m-%d").to_string());
                self.todo_state.items.push(TodoItem {
                    id:           task.id.to_string(),
                    title:        task.title,
                    description:  task.description.unwrap_or_default(),
                    status:       TodoStatus::from_str(&task.status),
                    priority:     TodoPriority::from_str(&task.priority),
                    deadline:     deadline_display,
                    project_name: None,
                    created_at:   task.created_at.format("%Y-%m-%d %H:%M").to_string(),
                });

                self.todo_state.form_title.clear();
                self.todo_state.form_description.clear();
                self.todo_state.form_priority = TodoPriority::Medium;
                self.todo_state.form_deadline.clear();
                self.todo_state.form_project.clear();
                self.todo_state.form_error = None;
                self.todo_state.show_form = false;
                true
            }
            Err(e) => {
                self.todo_state.form_error = Some(format!("Failed to create task: {}", e));
                false
            }
        }
    }

    pub fn save_edit_todo_sync(&mut self) -> bool {
        let id = match &self.todo_state.editing_id.clone() {
            Some(i) => i.clone(),
            None => return false,
        };

        let title = self.todo_state.form_title.trim().to_string();
        if title.is_empty() {
            self.todo_state.form_error = Some("Title cannot be empty".to_string());
            return false;
        }

        let token = match &self.token { Some(t) => t.clone(), None => return false };

        let deadline_str = if self.todo_state.form_deadline.trim().is_empty() {
            None
        } else {
            Some(self.todo_state.form_deadline.trim())
        };

        let priority_str = match self.todo_state.form_priority {
            TodoPriority::Low    => "low",
            TodoPriority::Medium => "medium",
            TodoPriority::High   => "high",
        };

        let status_str = if let Some(item) = self.todo_state.items.iter().find(|i| i.id == id) {
            match item.status {
                TodoStatus::Todo       => "todo",
                TodoStatus::InProgress => "in_progress",
                TodoStatus::Done       => "done",
            }
        } else {
            "todo"
        };

        let description = if self.todo_state.form_description.trim().is_empty() {
            None
        } else {
            Some(self.todo_state.form_description.trim())
        };

        match self.api_client.update_personal_task_sync(
            &id,
            Some(&title),
            description,
            Some(status_str),
            Some(priority_str),
            deadline_str,
            &token,
        ) {
            Ok(task) => {
                let deadline_display = task.deadline.map(|dt| dt.format("%Y-%m-%d").to_string());
                if let Some(item) = self.todo_state.items.iter_mut().find(|i| i.id == id) {
                    item.title = task.title;
                    item.description = task.description.unwrap_or_default();
                    item.priority = TodoPriority::from_str(&task.priority);
                    item.status = TodoStatus::from_str(&task.status);
                    item.deadline = deadline_display;
                }
                self.cancel_todo_form();
                true
            }
            Err(e) => {
                self.todo_state.form_error = Some(format!("Failed to update: {}", e));
                false
            }
        }
    }

    pub fn start_edit_todo(&mut self, id: &str) {
        if let Some(item) = self.todo_state.items.iter().find(|i| i.id == id) {
            self.todo_state.form_title       = item.title.clone();
            self.todo_state.form_description = item.description.clone();
            self.todo_state.form_priority    = item.priority.clone();
            self.todo_state.form_deadline    = item.deadline.clone().unwrap_or_default();
            self.todo_state.form_project     = item.project_name.clone().unwrap_or_default();
            self.todo_state.editing_id       = Some(id.to_string());
            self.todo_state.show_form        = true;
            self.todo_state.form_error       = None;
        }
    }

    pub fn cancel_todo_form(&mut self) {
        self.todo_state.show_form        = false;
        self.todo_state.editing_id       = None;
        self.todo_state.form_title.clear();
        self.todo_state.form_description.clear();
        self.todo_state.form_priority    = TodoPriority::Medium;
        self.todo_state.form_deadline.clear();
        self.todo_state.form_project.clear();
        self.todo_state.form_error       = None;
    }

    pub fn delete_todo_sync(&mut self, id: &str) {
        let token = match &self.token { Some(t) => t.clone(), None => return };

        match self.api_client.delete_personal_task_sync(id, &token) {
            Ok(_) => {
                self.todo_state.items.retain(|i| i.id != id);
                self.todo_state.confirm_delete_id = None;
            }
            Err(e) => {
                eprintln!("⚠️ Failed to delete task: {}", e);
            }
        }
    }

    pub fn toggle_todo_done_sync(&mut self, id: &str) {
        let token = match &self.token { Some(t) => t.clone(), None => return };

        if let Some(item) = self.todo_state.items.iter_mut().find(|i| i.id == id) {
            let new_status = if item.status == TodoStatus::Done {
                "todo"
            } else {
                "done"
            };

            match self.api_client.update_personal_task_sync(
                &id,
                None,
                None,
                Some(new_status),
                None,
                None,
                &token,
            ) {
                Ok(_) => {
                    item.status = if item.status == TodoStatus::Done {
                        TodoStatus::Todo
                    } else {
                        TodoStatus::Done
                    };
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to toggle task: {}", e);
                }
            }
        }
    }

    pub fn load_calendar_tasks_sync(&mut self) {
        let token = match &self.token { Some(t) => t.clone(), None => return };

        match self.api_client.get_personal_tasks_with_deadline_sync(&token) {
            Ok(tasks) => {
                self.todo_state.items = tasks.into_iter().map(|task| {
                    let deadline_str = task.deadline.map(|dt| dt.format("%Y-%m-%d").to_string());
                    TodoItem {
                        id:           task.id.to_string(),
                        title:        task.title,
                        description:  task.description.unwrap_or_default(),
                        status:       TodoStatus::from_str(&task.status),
                        priority:     TodoPriority::from_str(&task.priority),
                        deadline:     deadline_str,
                        project_name: None,
                        created_at:   task.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    }
                }).collect();
            }
            Err(e) => {
                eprintln!("⚠️ Failed to load calendar tasks: {}", e);
            }
        }
    }

    // ─── PROFILE METHODS ──────────────────────────────────────────────────────

    pub fn load_profile_sync(&mut self) {
        if let Some(user) = &self.current_user {
            let user_id = user.id.to_string();
            let token = match &self.token { Some(t) => t.clone(), None => return };

            match self.api_client.get_user_sync(&user_id, &token) {
                Ok(user_data) => {
                    self.current_user = Some(user_data);
                    self.profile_state.edit_name = self.current_user.as_ref().map(|u| u.name.clone()).unwrap_or_default();
                    self.profile_state.edit_email = self.current_user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to load profile: {}", e);
                }
            }
        }
    }

    pub fn start_edit_profile(&mut self) {
        if let Some(user) = &self.current_user {
            self.profile_state.edit_name  = user.name.clone();
            self.profile_state.edit_email = user.email.clone();
        }
        self.profile_state.is_editing  = true;
        self.profile_state.success_msg = None;
        self.profile_state.error_msg   = None;
    }

    pub fn save_profile_sync(&mut self) {
        let name = self.profile_state.edit_name.trim().to_string();
        if name.is_empty() {
            self.profile_state.error_msg = Some("Name cannot be empty".to_string());
            return;
        }

        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return };

        match self.api_client.update_user_sync(&user_id, &name, &token) {
            Ok(user_data) => {
                self.current_user = Some(user_data);
                self.profile_state.is_editing  = false;
                self.profile_state.success_msg = Some("✅ Profile updated successfully".to_string());
                self.profile_state.error_msg   = None;
                self.profile_state.msg_time    = std::time::Instant::now();
            }
            Err(e) => {
                self.profile_state.error_msg = Some(format!("Failed to update: {}", e));
            }
        }
    }

    pub fn cancel_edit_profile(&mut self) {
        self.profile_state.is_editing = false;
        self.profile_state.success_msg = None;
        self.profile_state.error_msg = None;
    }

    pub fn update_user_role_sync(&mut self, new_role: &str) -> bool {
        let user_id = match &self.current_user { Some(u) => u.id.to_string(), None => return false };
        let token   = match &self.token        { Some(t) => t.clone(),        None => return false };

        match self.api_client.update_user_role_sync(&user_id, new_role, &token) {
            Ok(user_data) => {
                self.current_user = Some(user_data);
                self.profile_state.success_msg = Some(format!("✅ Role updated to {}", new_role));
                self.profile_state.msg_time = std::time::Instant::now();
                true
            }
            Err(e) => {
                self.profile_state.error_msg = Some(format!("Failed to update role: {}", e));
                false
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  PARSERS
// ─────────────────────────────────────────────────────────────────────────────

fn parse_notifications(response: &serde_json::Value) -> Vec<Notification> {
    let data = match response.get("data").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    data.iter().filter_map(|item| {
        let id       = item.get("id")?.as_str()?.to_string();
        let title    = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let message  = item.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let type_str = item.get("notification_type").and_then(|v| v.as_str()).unwrap_or("in_app");
        let stat_str = item.get("status").and_then(|v| v.as_str()).unwrap_or("sent");
        let created  = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let created_at = if created.len() >= 16 {
            created[..16].replace('T', " ")
        } else {
            created.clone()
        };

        Some(Notification {
            id,
            title,
            message,
            notif_type: NotifType::from_str(type_str),
            status:     NotifStatus::from_str(stat_str),
            created_at,
        })
    }).collect()
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
            id:       format!("INV-{}", &id[..8.min(id.len())].to_uppercase()),
            date,
            plan,
            amount,
            currency,
            status,
        })
    }).collect()
}