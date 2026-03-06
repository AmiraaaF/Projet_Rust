use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "task_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[sqlx(type_name = "task_priority", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]

pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
//structure de la table task 
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: Uuid,
    pub assignee_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
}

// #[derive(Debug, Deserialize)]
// pub struct UpdateTaskRequest {
//     pub assignee_id: Option<Uuid>,
//     pub title: Option<String>,
//     pub description: Option<String>,
//     pub status: Option<String>,
//     pub priority: Option<String>,
//     pub deadline: Option<DateTime<Utc>>,
// }

#[derive(Debug, Deserialize)]

//filtre et recherche par titre des taches 
pub struct TaskFilters {
    pub project_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub search: Option<String>,
}
