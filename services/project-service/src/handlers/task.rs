use axum::Json;
use serde_json::json;

pub async fn create_task() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn list_tasks() -> Json<serde_json::Value> {
    Json(json!([
        {
            "id": "1",
            "title": "Design new landing page",
            "status": "Todo",
            "priority": "medium",
            "assignee": "Sarah M.",
            "tags": ["design", "frontend"],
            "due_date": "Mar 5"
        },
        {
            "id": "2",
            "title": "Implement OAuth2 login",
            "status": "InProgress",
            "priority": "high",
            "assignee": "John D.",
            "tags": ["backend", "auth"],
            "due_date": "Mar 3"
        },
        {
            "id": "3",
            "title": "Database schema migration",
            "status": "Done",
            "priority": "high",
            "assignee": "Sarah M.",
            "tags": ["backend"],
            "due_date": "Feb 27"
        }
    ]))
}
pub async fn get_task() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn update_task() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn delete_task() -> Json<serde_json::Value> {
    Json(json!({"message": "deleted"}))
}
