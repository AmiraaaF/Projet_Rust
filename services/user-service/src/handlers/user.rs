use axum::Json;
use serde_json::json;

pub async fn list_users() -> Json<serde_json::Value> {
    Json(json!([]))
}

pub async fn get_user() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn update_user() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn delete_user() -> Json<serde_json::Value> {
    Json(json!({"message": "deleted"}))
}
