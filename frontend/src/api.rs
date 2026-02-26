use shared::models::*;
use reqwest::Client;

pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn register(&self, req: &RegisterRequest) -> Result<AuthResponse, String> {
        self.client
            .post(&format!("{}/auth/register", self.base_url))
            .header("Content-Type", "application/json")
            .json(req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, String> {
        self.client
            .post(&format!("{}/auth/login", self.base_url))
            .header("Content-Type", "application/json")
            .json(req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_users(&self, page: i64, limit: i64) -> Result<PaginatedResponse<UserPublic>, String> {
        self.client
            .get(&format!(
                "{}/users?page={}&limit={}",
                self.base_url, page, limit
            ))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_project(&self, req: &CreateProjectRequest) -> Result<Project, String> {
        self.client
            .post(&format!("{}/projects", self.base_url))
            .header("Content-Type", "application/json")
            .json(req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_projects(&self, page: i64, limit: i64) -> Result<PaginatedResponse<Project>, String> {
        self.client
            .get(&format!(
                "{}/projects?page={}&limit={}",
                self.base_url, page, limit
            ))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_project(&self, project_id: &str) -> Result<Project, String> {
        self.client
            .get(&format!("{}/projects/{}", self.base_url, project_id))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_tasks(&self, project_id: &str, page: i64, limit: i64) -> Result<PaginatedResponse<Task>, String> {
        self.client
            .get(&format!(
                "{}/projects/{}/tasks?page={}&limit={}",
                self.base_url, project_id, page, limit
            ))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn create_task(&self, project_id: &str, req: &CreateTaskRequest) -> Result<Task, String> {
        self.client
            .post(&format!("{}/projects/{}/tasks", self.base_url, project_id))
            .header("Content-Type", "application/json")
            .json(req)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())
    }
}
