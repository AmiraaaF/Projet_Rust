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

use crate::models::{all_plans, get_plan_info};
use shared::{CreateInvoiceRequest, CreateSubscriptionRequest, UpdateSubscriptionRequest};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

// ─────────────────────────────────────────────────────────────────────────────
//  PLANS
// ─────────────────────────────────────────────────────────────────────────────

pub async fn list_plans() -> Json<Value> {
    Json(json!(all_plans()))
}

pub async fn get_plan(Path(plan_id): Path<String>) -> Json<Value> {
    Json(json!(get_plan_info(&plan_id)))
}

// ─────────────────────────────────────────────────────────────────────────────
//  SUBSCRIPTIONS
// ─────────────────────────────────────────────────────────────────────────────

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
        VALUES ($1, ($2)::subscription_plan, 'active'::subscription_status, NOW(), $3, true, $4, $5)
        ON CONFLICT (user_id) DO UPDATE SET
            plan         = EXCLUDED.plan,
            status       = 'active'::subscription_status,
            started_at   = NOW(),
            expires_at   = EXCLUDED.expires_at,
            max_projects = EXCLUDED.max_projects,
            max_tasks    = EXCLUDED.max_tasks,
            auto_renew   = true,
            updated_at   = NOW()
        RETURNING id, user_id, CAST(plan AS TEXT), CAST(status AS TEXT), auto_renew, max_projects, max_tasks
        "#,
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
            let auto_renew: bool = row.get(4);
            let max_projects: i32 = row.get(5);
            let max_tasks: i32 = row.get(6);

            if plan_info.price_monthly > 0.0 {
                let _ = sqlx::query(
                    r#"
                    INSERT INTO invoices
                        (user_id, subscription_id, amount, currency, status, issued_at, due_date)
                    VALUES ($1, $2, $3, 'USD', 'paid'::invoice_status, NOW(), NOW() + INTERVAL '30 days')
                    "#,
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
        SELECT id, user_id, CAST(plan AS TEXT), CAST(status AS TEXT), auto_renew, max_projects, max_tasks
        FROM subscriptions
        WHERE user_id = $1
        "#,
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
                "max_projects": row.get::<Option<i32>, _>(5),
                "max_tasks": row.get::<Option<i32>, _>(6),
                "plan_info": plan_info,
            })))
        }
        Ok(None) => {
            Ok(Json(json!({
                "user_id": user_id,
                "plan": "free",
                "status": "active",
                "auto_renew": false,
                "max_projects": 3,
                "max_tasks": 100,
                "plan_info": get_plan_info("free"),
            })))
        }
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
    if let Some(ref plan) = body.plan {
        let valid_plans = ["free", "starter", "pro", "enterprise"];
        if !valid_plans.contains(&plan.as_str()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("Plan '{}' invalide. Valeurs acceptées: free, starter, pro, enterprise", plan)})),
            ));
        }
    }

    let plan_str = body.plan.as_deref().unwrap_or("free");
    let plan_info = get_plan_info(plan_str);
    let expires_at = if plan_str == "free" {
        None::<chrono::DateTime<Utc>>
    } else {
        Some(Utc::now() + chrono::Duration::days(30))
    };


    let status_str = body.status.as_deref().unwrap_or("active");
    let auto_renew = body.auto_renew.unwrap_or(true);

    match sqlx::query(
        r#"
        INSERT INTO subscriptions
            (user_id, plan, status, started_at, expires_at, auto_renew, max_projects, max_tasks)
        VALUES
            ($1, $2::subscription_plan, $3::subscription_status, NOW(), $4, $5, $6, $7)
        ON CONFLICT (user_id) DO UPDATE SET
            plan         = $2::subscription_plan,
            status       = $3::subscription_status,
            expires_at   = $4,
            auto_renew   = $5,
            max_projects = $6,
            max_tasks    = $7,
            updated_at   = NOW()
        RETURNING id, user_id, CAST(plan AS TEXT), CAST(status AS TEXT), auto_renew, max_projects, max_tasks, updated_at
        "#,
    )
    .bind(user_id)
    .bind(plan_str)            
    .bind(status_str)          
    .bind(expires_at)          
    .bind(auto_renew)          
    .bind(plan_info.max_projects) 
    .bind(plan_info.max_tasks)    
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => {
            let subscription_id: Uuid = row.get(0);
            let plan_name: String = row.get(2);
            
            // Créer une invoice si c'est un plan payant
            if plan_info.price_monthly > 0.0 {
                let _ = sqlx::query(
                    r#"
                    INSERT INTO invoices
                        (user_id, subscription_id, amount, currency, status, issued_at, due_date)
                    VALUES ($1, $2, $3, 'USD', 'paid'::invoice_status, NOW(), NOW() + INTERVAL '30 days')
                    "#,
                )
                .bind(user_id)
                .bind(subscription_id)
                .bind(plan_info.price_monthly)
                .execute(&state.db)
                .await;
            }
            
            Ok(Json(json!({
                "id": subscription_id,
                "user_id": row.get::<Uuid, _>(1),
                "plan": plan_name,
                "status": row.get::<String, _>(3),
                "auto_renew": row.get::<bool, _>(4),
                "max_projects": row.get::<i32, _>(5),
                "max_tasks": row.get::<i32, _>(6),
                "updated_at": row.get::<chrono::DateTime<Utc>, _>(7),
                "plan_info": get_plan_info(&plan_name),
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Erreur mise à jour: {}", e)})),
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
        SET plan = 'free'::subscription_plan, status = 'active'::subscription_status, auto_renew = false, 
            expires_at = NULL, max_projects = 3, max_tasks = 100, updated_at = NOW()
        WHERE user_id = $1
        RETURNING id, CAST(plan AS TEXT), CAST(status AS TEXT)
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => Ok(Json(json!({
            "message": "Subscription cancelled, reverted to free",
            "id": row.get::<Uuid, _>(0),
            "plan": row.get::<String, _>(1),
            "status": row.get::<String, _>(2),
        }))),
        Err(e) => {
            // Si pas de subscription, on en crée une free
            match sqlx::query(
                r#"
                INSERT INTO subscriptions
                    (user_id, plan, status, started_at, expires_at, auto_renew, max_projects, max_tasks)
                VALUES ($1, 'free'::subscription_plan, 'active'::subscription_status, NOW(), NULL, false, 3, 100)
                RETURNING id, CAST(plan AS TEXT), CAST(status AS TEXT)
                "#,
            )
            .bind(user_id)
            .fetch_one(&state.db)
            .await
            {
                Ok(row) => Ok(Json(json!({
                    "message": "Subscription cancelled and reset to free",
                    "id": row.get::<Uuid, _>(0),
                    "plan": row.get::<String, _>(1),
                    "status": row.get::<String, _>(2),
                }))),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Erreur: {}", e)})),
                )),
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  QUOTA
// ─────────────────────────────────────────────────────────────────────────────

pub async fn check_quota(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        "SELECT CAST(plan AS TEXT), CAST(status AS TEXT), max_projects, max_tasks FROM subscriptions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => {
            let status: String = row.get(1);
            Ok(Json(json!({
                "user_id": user_id,
                "plan": row.get::<String, _>(0),
                "subscription_active": status == "active",
                "quotas": {
                    "max_projects": row.get::<Option<i32>, _>(2).unwrap_or(3),
                    "max_tasks": row.get::<Option<i32>, _>(3).unwrap_or(100),
                }
            })))
        }
        Ok(None) => Ok(Json(json!({
            "user_id": user_id,
            "plan": "free",
            "subscription_active": false,
            "quotas": { "max_projects": 3, "max_tasks": 100 }
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  INVOICES
// ─────────────────────────────────────────────────────────────────────────────

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

    // Count total
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoices WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    match sqlx::query(
        r#"
        SELECT
            i.id,
            i.user_id,
            i.subscription_id,
            i.amount,
            i.currency,
            CAST(i.status AS TEXT),
            i.issued_at,
            i.due_date,
            i.paid_at,
            CASE WHEN s.plan IS NULL THEN 'free' ELSE CAST(s.plan AS TEXT) END AS plan
        FROM invoices i
        LEFT JOIN subscriptions s ON s.id = i.subscription_id
        WHERE i.user_id = $1
        ORDER BY i.issued_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    {
        Ok(rows) => {
            let data: Vec<Value> = rows
                .iter()
                .map(|r| {
                    json!({
                        "id":              r.get::<Uuid, _>(0),
                        "user_id":         r.get::<Uuid, _>(1),
                        "subscription_id": r.get::<Option<Uuid>, _>(2),
                        "amount":          r.get::<f64, _>(3),
                        "currency":        r.get::<String, _>(4),
                        "status":          r.get::<String, _>(5),
                        "issued_at":       r.get::<chrono::DateTime<Utc>, _>(6),
                        "due_date":        r.get::<Option<chrono::DateTime<Utc>>, _>(7),
                        "paid_at":         r.get::<Option<chrono::DateTime<Utc>>, _>(8),
                        "plan":            r.get::<String, _>(9),
                    })
                })
                .collect();

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

pub async fn get_invoice(
    State(state): State<AppState>,
    Path(invoice_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match sqlx::query(
        "SELECT id, user_id, amount, currency, CAST(status AS TEXT), issued_at, due_date, paid_at FROM invoices WHERE id = $1",
    )
    .bind(invoice_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Ok(Json(json!({
            "id":       row.get::<Uuid, _>(0),
            "user_id":  row.get::<Uuid, _>(1),
            "amount":   row.get::<f64, _>(2),
            "currency": row.get::<String, _>(3),
            "status":   row.get::<String, _>(4),
            "issued_at": row.get::<chrono::DateTime<Utc>, _>(5),
            "due_date": row.get::<Option<chrono::DateTime<Utc>>, _>(6),
            "paid_at":  row.get::<Option<chrono::DateTime<Utc>>, _>(7),
        }))),
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
    let currency = body.currency.clone().unwrap_or_else(|| "USD".to_string());

    match sqlx::query(
        r#"
        INSERT INTO invoices
            (user_id, subscription_id, amount, currency, status, issued_at, due_date)
        VALUES ($1, $2, $3, $4, 'paid'::invoice_status, NOW(), $5)
        RETURNING id, user_id, amount, currency, CAST(status AS TEXT), issued_at
        "#,
    )
    .bind(body.user_id)
    .bind(body.subscription_id)
    .bind(body.amount)
    .bind(&currency)
    .bind(body.due_date)
    .fetch_one(&state.db)
    .await
    {
        Ok(row) => Ok((
            StatusCode::CREATED,
            Json(json!({
                "id":       row.get::<Uuid, _>(0),
                "user_id":  row.get::<Uuid, _>(1),
                "amount":   row.get::<f64, _>(2),
                "currency": row.get::<String, _>(3),
                "status":   row.get::<String, _>(4),
                "issued_at": row.get::<chrono::DateTime<Utc>, _>(5),
            })),
        )),
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
        SET status = 'paid'::invoice_status, paid_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND status != 'paid'::invoice_status
        RETURNING id, amount, currency, CAST(status AS TEXT), paid_at
        "#,
    )
    .bind(invoice_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Ok(Json(json!({
            "message":  "Invoice marked as paid",
            "id":       row.get::<Uuid, _>(0),
            "amount":   row.get::<f64, _>(1),
            "currency": row.get::<String, _>(2),
            "status":   row.get::<String, _>(3),
            "paid_at":  row.get::<Option<chrono::DateTime<Utc>>, _>(4),
        }))),
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
        "DELETE FROM invoices WHERE id = $1 AND status = 'draft'::invoice_status RETURNING id",
    )
    .bind(invoice_id)
    .fetch_optional(&state.db)
    .await
    {
        Ok(Some(row)) => Ok(Json(json!({
            "message": "Invoice deleted",
            "id": row.get::<Uuid, _>(0),
        }))),
        Ok(None) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invoice not found or cannot be deleted (only 'draft' invoices can be deleted)"})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )),
    }
}