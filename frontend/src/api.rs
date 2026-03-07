use serde::Deserialize;
use shared::models::*;

// Représente une tâche telle que renvoyée par le tasks-service
#[derive(Debug, Clone, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub project_id: String,
    pub assignee_id: Option<String>,
    pub title: String,
    pub description: Option<String>,  
    pub status: String,               
    pub priority: String,             
    pub deadline: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub assignee_name: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    billing_url: String,
    tasks_url: String,  
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        let billing_url = base_url
            .replace(":3001", ":3003")
            .replace(":3002", ":3003");

        // Le tasks-service tourne sur le port 3004
        let tasks_url = base_url
            .replace(":3001", ":3004")
            .replace(":3002", ":3004");
        Self { base_url, billing_url, tasks_url}
    }

    // ─── AUTH ──────────────────────────────────────────────────────────────────

    pub fn login_sync(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/auth/login", self.base_url);
        let body = serde_json::json!({ "email": email, "password": password });

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            resp.json::<AuthResponse>()
                .map_err(|e| format!("Réponse invalide du serveur: {}", e))
        } else {
            let text = resp.text().unwrap_or_default();
            let msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("Erreur serveur ({})", status));
            Err(msg)
        }
    }

    pub fn register_sync(&self, email: &str, name: &str, password: &str) -> Result<AuthResponse, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/auth/register", self.base_url);
        let body = serde_json::json!({ "email": email, "name": name, "password": password });

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            resp.json::<AuthResponse>()
                .map_err(|e| format!("Réponse invalide: {}", e))
        } else {
            let text = resp.text().unwrap_or_default();
            let msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("Erreur serveur ({})", status));
            Err(msg)
        }
    }

    // ─── USERS ─────────────────────────────────────────────────────────────────

    pub fn get_users_sync(&self, page: i64, limit: i64) -> Result<PaginatedResponse<UserPublic>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/users?page={}&limit={}", self.base_url, page, limit);
        client.get(&url).send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    // ─── PROJECTS ──────────────────────────────────────────────────────────────

    pub fn create_project_sync(&self, name: &str, description: Option<&str>, token: &str) -> Result<Project, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects", self.base_url);
        let body = serde_json::json!({ "name": name, "description": description });

        client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_projects_sync(&self, page: i64, limit: i64, token: &str) -> Result<PaginatedResponse<Project>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects?page={}&limit={}", self.base_url, page, limit);
        client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_project_sync(&self, project_id: &str, token: &str) -> Result<Project, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects/{}", self.base_url, project_id);
        client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_tasks_sync(&self, project_id: &str, page: i64, limit: i64, token: &str) -> Result<PaginatedResponse<Task>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects/{}/tasks?page={}&limit={}", self.base_url, project_id, page, limit);
        client.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn create_task_sync(&self, project_id: &str, title: &str, description: Option<&str>, token: &str) -> Result<Task, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects/{}/tasks", self.base_url, project_id);
        let body = serde_json::json!({ "title": title, "description": description });
        client.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    // ─── BILLING ───────────────────────────────────────────────────────────────

    pub fn get_subscription_sync(&self, user_id: &str, token: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/billing/subscriptions/{}", self.billing_url, user_id);

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau billing: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            resp.json::<serde_json::Value>()
                .map_err(|e| format!("Réponse billing invalide: {}", e))
        } else {
            let text = resp.text().unwrap_or_default();
            Err(format!("Erreur billing ({}): {}", status, text))
        }
    }

    pub fn update_subscription_sync(&self, user_id: &str, plan: &str, token: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/billing/subscriptions/{}", self.billing_url, user_id);
        let body = serde_json::json!({ "plan": plan });

        let resp = client
            .patch(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau billing: {}", e))?;

        let status = resp.status();
        let text = resp.text().unwrap_or_default();

        if status.is_success() {
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|e| format!("Réponse billing invalide: {}", e))
        } else {
            // Retourner le message d'erreur du serveur
            let msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("Erreur serveur billing ({})", status));
            Err(msg)
        }
    }

    
    pub fn cancel_subscription_sync(&self, user_id: &str, token: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/billing/subscriptions/{}/cancel", self.billing_url, user_id);

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau billing: {}", e))?;

        let status = resp.status();
        let text = resp.text().unwrap_or_default();

        if status.is_success() {
            serde_json::from_str(&text).map_err(|e| format!("Réponse invalide: {}", e))
        } else {
            let msg = serde_json::from_str::<serde_json::Value>(&text)
                .ok()
                .and_then(|v| v["error"].as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("Erreur ({})", status));
            Err(msg)
        }
    }

    
    pub fn get_invoices_sync(&self, user_id: &str, token: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/billing/invoices/{}", self.billing_url, user_id);

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau billing: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            resp.json().map_err(|e| format!("Réponse invalide: {}", e))
        } else {
            Err(format!("Erreur billing ({})", status))
        }
    }

    // ─── TASKS ──────────────────────────────────────────────────────────────

    pub fn list_tasks_sync( &self, assignee_id: Option<&str>, status: Option<&str>,  project_id: Option<&str>, token: &str,) -> Result<Vec<TaskResponse>, String> {
        let client = reqwest::blocking::Client::new();

        let mut params: Vec<String> = Vec::new();
        if let Some(id) = assignee_id {
            params.push(format!("assignee_id={}", id));
        }
        if let Some(s) = status {
            params.push(format!("status={}", s));
        }
        let url = if params.is_empty() {
            format!("{}/tasks", self.tasks_url)
        } else {
            format!("{}/tasks?{}", self.tasks_url, params.join("&"))
        };

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau tasks: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            resp.json::<Vec<TaskResponse>>()
                .map_err(|e| format!("Réponse tasks invalide: {}", e))
        } else {
            let text = resp.text().unwrap_or_default();
            Err(format!("Erreur tasks ({}): {}", status, text))
        }
    }

    // Marque une tâche comme "done" 
    pub fn mark_task_done_sync(&self, task_id: &str, token: &str) -> Result<(), String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/tasks/{}/done", self.tasks_url, task_id);

        let resp = client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau tasks: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().unwrap_or_default();
            Err(format!("Erreur mark done ({}): {}", status, text))
        }
    }


    pub fn delete_task_sync(&self, task_id: &str, token: &str) -> Result<(), String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/tasks/{}", self.tasks_url, task_id);

        let resp = client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau tasks: {}", e))?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().unwrap_or_default();
            Err(format!("Erreur delete task ({}): {}", status, text))
        }
    }

  
    pub fn create_task_on_service_sync(
        &self,
        title: &str,
        description: Option<&str>,
        status: &str,
        priority: &str,
        assignee_id: Option<&str>,
        deadline: Option<&str>,
        project_id: Option<&str>,
        token: &str,
    ) -> Result<TaskResponse, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/tasks", self.tasks_url);

        // On construit le body JSON avec serde_json::json!
        // Les champs optionnels sont inclus seulement s'ils ont une valeur
        let body = serde_json::json!({
            "title":       title,
            "description": description,
            "status":      status,
            "priority":    priority,
            "assignee_id": assignee_id,
            "deadline":    deadline,
            "project_id":  project_id,
        });

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau tasks: {}", e))?;

        let status_code = resp.status();
        if status_code.is_success() {
            resp.json::<TaskResponse>()
                .map_err(|e| format!("Réponse tasks invalide: {}", e))
        } else {
            let text = resp.text().unwrap_or_default();
            Err(format!("Erreur création tâche ({}): {}", status_code, text))
        }
    }

}