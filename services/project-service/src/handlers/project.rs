use axum::Json;
use serde_json::json;

pub async fn create_project() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn list_projects() -> Json<serde_json::Value> {
    Json(json!([]))
}

pub async fn get_project() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn update_project() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn delete_project() -> Json<serde_json::Value> {
    Json(json!({"message": "deleted"}))
}

pub async fn get_members() -> Json<serde_json::Value> {
    Json(json!([]))
}

pub async fn add_member() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn remove_member() -> Json<serde_json::Value> {
    Json(json!({"message": "removed"}))
}
