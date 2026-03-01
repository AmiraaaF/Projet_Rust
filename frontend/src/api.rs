use shared::models::*;

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
        }
    }


    pub fn login_sync(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/auth/login", self.base_url);
        
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });

        client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json::<AuthResponse>()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    /// Synchronous register - blocks until response
    pub fn register_sync(&self, email: &str, name: &str, password: &str) -> Result<AuthResponse, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/auth/register", self.base_url);
        
        let body = serde_json::json!({
            "email": email,
            "name": name,
            "password": password,
        });

        client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json::<AuthResponse>()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_users_sync(&self, page: i64, limit: i64) -> Result<PaginatedResponse<UserPublic>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "{}/users?page={}&limit={}",
            self.base_url, page, limit
        );
        
        client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn create_project_sync(&self, name: &str, description: Option<&str>, token: &str) -> Result<Project, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects", self.base_url);
        
        let body = serde_json::json!({
            "name": name,
            "description": description,
        });

        client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_projects_sync(&self, page: i64, limit: i64, token: &str) -> Result<PaginatedResponse<Project>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "{}/projects?page={}&limit={}",
            self.base_url, page, limit
        );
        
        client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_project_sync(&self, project_id: &str, token: &str) -> Result<Project, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects/{}", self.base_url, project_id);
        
        client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_tasks_sync(&self, project_id: &str, page: i64, limit: i64, token: &str) -> Result<PaginatedResponse<Task>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "{}/projects/{}/tasks?page={}&limit={}",
            self.base_url, project_id, page, limit
        );
        
        client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn create_task_sync(&self, project_id: &str, title: &str, description: Option<&str>, token: &str) -> Result<Task, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/projects/{}/tasks", self.base_url, project_id);
        
        let body = serde_json::json!({
            "title": title,
            "description": description,
        });

        client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn update_subscription_sync(&self, user_id: &str, plan: &str, token: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:3003/billing/subscriptions/{}", user_id);
        
        let body = serde_json::json!({
            "plan": plan,
        });

        client
            .patch(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .json(&body)
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }

    pub fn get_subscription_sync(&self, user_id: &str, token: &str) -> Result<serde_json::Value, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://localhost:3003/billing/subscriptions/{}", user_id);
        
        client
            .get(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .map_err(|e| format!("Erreur réseau: {}", e))?
            .json()
            .map_err(|e| format!("Réponse invalide: {}", e))
    }
}
