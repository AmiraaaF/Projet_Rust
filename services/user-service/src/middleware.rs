use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use shared::errors::AppError;

use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid token format".to_string()))?;

    let claims = state.auth.validate_token(token)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
