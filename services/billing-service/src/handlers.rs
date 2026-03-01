use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    all_plans, get_plan_info,
};
use shared::{CreateInvoiceRequest, CreateSubscriptionRequest, UpdateSubscriptionRequest};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// PLANS

pub async fn list_plans() -> Json<Value> {
    Json(json!(all_plans()))
}

pub async fn get_plan(Path(plan_id): Path<String>) -> Json<Value> {
    Json(json!(get_plan_info(&plan_id)))
}


// SUBSCRIPTIONS

pub async fn create_subscription(
    State(state): State<AppState>,
    Json(body): Json<CreateSubscriptionRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let plan = body.plan.as_str();
    let valid_plans = ["free", "starter", "pro", "enterprise"];
    if !valid_plans.contains(&plan) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Plan '{}' invalide", plan)})),
        ));
    }

    let plan_info = get_plan_info(plan);
    let expires_at = if plan == "free" {
        None
    } else {
        Some(Utc::now() + chrono::Duration::days(30))
    };

    match sqlx::query(
        r#"
        INSERT INTO subscriptions
            (user_id, plan, status, started_at, expires_at, auto_renew, max_projects, max_tasks)
        VALUES ($1, ($2)::subscription_plan, ('active')::subscription_status, NOW(), $3, true, $4, $5)
        ON CONFLICT (user_id) DO UPDATE SET
            plan = EXCLUDED.plan,
            status = 'active'::subscription_status,
            started_at = NOW(),
            expires_at = EXCLUDED.expires_at,
            max_projects = EXCLUDED.max_projects,
            max_tasks = EXCLUDED.max_tasks,
            auto_renew = true,
            updated_at = NOW()
        RETURNING id, user_id, plan::text, status::text, started_at, expires_at,
                  auto_renew, max_projects, max_tasks, created_at, updated_at
        "#
    )
    .bind(body.user_id)
    .bind(&body.plan)
    .bind(expires_at)
    .bind(plan_info.max_projects)
    .bind(plan_info.max_tasks)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => {
            let id: Uuid = row.get(0);
            let uid: Uuid = row.get(1);
            let plan_name: String = row.get(2);
            let status: String = row.get(3);
            let auto_renew: bool = row.get(6);
            let max_projects: i32 = row.get(7);
            let max_tasks: i32 = row.get(8);

            if plan_info.price_monthly > 0.0 {
                let _ = sqlx::query(
                    r#"
                    INSERT INTO invoices
                        (user_id, subscription_id, amount, currency, status, issued_at, due_date)
                    VALUES ($1, $2, $3, 'USD', 'issued'::invoice_status, NOW(), NOW() + INTERVAL '30 days')
                    "#
                )
                .bind(body.user_id)
                .bind(id)
                .bind(plan_info.price_monthly)
                .execute(&state.db)
                .await;
            }

            Ok((
                StatusCode::CREATED,
                Json(json!({
                    "id": id,
                    "user_id": uid,
                    "plan": plan_name,
                    "status": status,
                    "auto_renew": auto_renew,
                    "max_projects": max_projects,
                    "max_tasks": max_tasks,
                    "plan_info": plan_info,
                })),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_subscription(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        SELECT id, user_id, plan, status, auto_renew, max_projects, max_tasks
        FROM subscriptions
        WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            let plan: String = row.get(2);
            let plan_info = get_plan_info(&plan);
            Ok(Json(json!({
                "user_id": row.get::<Uuid, _>(1),
                "plan": plan,
                "status": row.get::<String, _>(3),
                "auto_renew": row.get::<bool, _>(4),
                "max_projects": row.get::<i32, _>(5),
                "max_tasks": row.get::<i32, _>(6),
                "plan_info": plan_info,
            })))
        }
        Ok(None) => Ok(Json(json!({
            "user_id": user_id,
            "plan": "free",
            "status": "active",
            "auto_renew": false,
            "max_projects": 3,
            "max_tasks": 100,
            "plan_info": get_plan_info("free"),
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn update_subscription(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UpdateSubscriptionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validation du plan s'il est fourni
    if let Some(ref plan) = body.plan {
        let valid_plans = ["free", "starter", "pro", "enterprise"];
        if !valid_plans.contains(&plan.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("Plan '{}' invalide", plan)})),
            ));
        }
    }

    // Récupérer les infos du plan si fourni
    let (new_plan, new_max_projects, new_max_tasks) = if let Some(plan) = &body.plan {
        let info = get_plan_info(plan);
        (Some(plan.clone()), Some(info.max_projects), Some(info.max_tasks))
    } else {
        (None, None, None)
    };

    match sqlx::query(
        r#"
        UPDATE subscriptions SET
            plan = CASE WHEN $2::text IS NOT NULL THEN CAST($2 AS subscription_plan) ELSE plan END,
            status = CASE WHEN $3::text IS NOT NULL THEN CAST($3 AS subscription_status) ELSE status END,
            auto_renew = CASE WHEN $4::boolean IS NOT NULL THEN $4 ELSE auto_renew END,
            max_projects = CASE WHEN $5::integer IS NOT NULL THEN $5 ELSE max_projects END,
            max_tasks = CASE WHEN $6::integer IS NOT NULL THEN $6 ELSE max_tasks END,
            updated_at = NOW()
        WHERE user_id = $1
        RETURNING id, user_id, plan::text, status::text, auto_renew, max_projects, max_tasks
        "#
    )
    .bind(user_id)
    .bind(new_plan)
    .bind(&body.status)
    .bind(body.auto_renew)
    .bind(new_max_projects)
    .bind(new_max_tasks)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            let plan: String = row.get(2);
            Ok(Json(json!({
                "id": row.get::<Uuid, _>(0),
                "user_id": row.get::<Uuid, _>(1),
                "plan": plan,
                "status": row.get::<String, _>(3),
                "auto_renew": row.get::<bool, _>(4),
                "max_projects": row.get::<i32, _>(5),
                "max_tasks": row.get::<i32, _>(6),
                "plan_info": get_plan_info(&plan),
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Subscription not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn cancel_subscription(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        UPDATE subscriptions
        SET status = 'cancelled'::subscription_status, auto_renew = false, updated_at = NOW()
        WHERE user_id = $1 AND status = 'active'::subscription_status
        RETURNING id, plan::text, status::text
        "#
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            Ok(Json(json!({
                "message": "Subscription cancelled",
                "id": row.get::<Uuid, _>(0),
                "plan": row.get::<String, _>(1),
                "status": row.get::<String, _>(2),
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Active subscription not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}


// QUOTA

pub async fn check_quota(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        "SELECT plan, status, max_projects, max_tasks FROM subscriptions WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            let status: String = row.get(1);
            let active = status == "active";
            Ok(Json(json!({
                "user_id": user_id,
                "plan": row.get::<String, _>(0),
                "subscription_active": active,
                "quotas": {
                    "max_projects": row.get::<i32, _>(2),
                    "max_tasks": row.get::<i32, _>(3),
                }
            })))
        }
        Ok(None) => Ok(Json(json!({
            "user_id": user_id,
            "plan": "free",
            "subscription_active": false,
            "quotas": {
                "max_projects": 3,
                "max_tasks": 100,
            }
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}


// INVOICES

#[derive(Deserialize)]
pub struct InvoiceQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
}

pub async fn list_invoices(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(q): Query<InvoiceQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(10).min(100);
    let offset = (page - 1) * limit;

    match sqlx::query(
        r#"
        SELECT COUNT(*) FROM invoices WHERE user_id = $1
        "#
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(total_row) => {
            let total: i64 = total_row.get(0);
            match sqlx::query(
                r#"
                SELECT id, user_id, amount, currency, status, issued_at
                FROM invoices
                WHERE user_id = $1
                ORDER BY issued_at DESC
                LIMIT $2 OFFSET $3
                "#
            )
            .bind(user_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            {
                Ok(rows) => {
                    let data: Vec<Value> = rows.iter().map(|r| {
                        json!({
                            "id": r.get::<Uuid, _>(0),
                            "user_id": r.get::<Uuid, _>(1),
                            "amount": r.get::<f64, _>(2),
                            "currency": r.get::<String, _>(3),
                            "status": r.get::<String, _>(4),
                            "issued_at": r.get::<chrono::DateTime<chrono::Utc>, _>(5),
                        })
                    }).collect();
                    Ok(Json(json!({
                        "data": data,
                        "page": page,
                        "limit": limit,
                        "total": total,
                    })))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )),
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        "SELECT id, user_id, amount, currency, status, issued_at FROM invoices WHERE id = $1"
    )
    .bind(invoice_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            Ok(Json(json!({
                "id": row.get::<Uuid, _>(0),
                "user_id": row.get::<Uuid, _>(1),
                "amount": row.get::<f64, _>(2),
                "currency": row.get::<String, _>(3),
                "status": row.get::<String, _>(4),
                "issued_at": row.get::<chrono::DateTime<chrono::Utc>, _>(5),
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Invoice not found"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn create_invoice(
    State(state): State<AppState>,
    Json(body): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    if body.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Amount must be positive"})),
        ));
    }
    let currency = body.currency.unwrap_or_else(|| "USD".to_string());

    match sqlx::query(
        r#"
        INSERT INTO invoices
            (user_id, subscription_id, amount, currency, status, issued_at, due_date)
        VALUES ($1, $2, $3, $4, 'issued'::invoice_status, NOW(), $5)
        RETURNING id, user_id, amount, currency, status, issued_at
        "#
    )
    .bind(body.user_id)
    .bind(body.subscription_id)
    .bind(body.amount)
    .bind(&currency)
    .bind(body.due_date)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => {
            Ok((
                StatusCode::CREATED,
                Json(json!({
                    "id": row.get::<Uuid, _>(0),
                    "user_id": row.get::<Uuid, _>(1),
                    "amount": row.get::<f64, _>(2),
                    "currency": row.get::<String, _>(3),
                    "status": row.get::<String, _>(4),
                    "issued_at": row.get::<chrono::DateTime<chrono::Utc>, _>(5),
                })),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn pay_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        r#"
        UPDATE invoices
        SET status = 'paid'::invoice_status, paid_at = NOW()
        WHERE id = $1 AND status != 'paid'::invoice_status
        RETURNING id, amount, currency, status
        "#
    )
    .bind(invoice_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            Ok(Json(json!({
                "message": "Invoice marked as paid",
                "id": row.get::<Uuid, _>(0),
                "amount": row.get::<f64, _>(1),
                "currency": row.get::<String, _>(2),
                "status": row.get::<String, _>(3),
            })))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Invoice not found or already paid"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

pub async fn delete_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        "DELETE FROM invoices WHERE id = $1 AND status = 'draft'::invoice_status RETURNING id"
    )
    .bind(invoice_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            Ok(Json(json!({
                "message": "Invoice deleted",
                "id": row.get::<Uuid, _>(0),
            })))
        }
        Ok(None) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invoice not found or cannot be deleted"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}
