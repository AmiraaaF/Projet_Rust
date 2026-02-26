use axum::Json;
use serde_json::json;

pub async fn register() -> Json<serde_json::Value> {
    Json(json!({"message": "register endpoint"}))
}

pub async fn login() -> Json<serde_json::Value> {
    Json(json!({"message": "login endpoint"}))
}
