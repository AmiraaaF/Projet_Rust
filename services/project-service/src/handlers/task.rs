use axum::Json;
use serde_json::json;

pub async fn create_task() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn list_tasks() -> Json<serde_json::Value> {
    Json(json!([]))
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
