use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanInfo {
    pub id: String,
    pub name: String,
    pub price_monthly: f64,
    pub max_projects: i32,
    pub max_tasks: i32,
    pub features: Vec<String>,
}

pub fn get_plan_info(plan: &str) -> PlanInfo {
    match plan {
        "starter" => PlanInfo {
            id: "starter".to_string(),
            name: "Starter".to_string(),
            price_monthly: 9.99,
            max_projects: 10,
            max_tasks: 500,
            features: vec![
                "10 projets max".to_string(),
                "500 tâches max".to_string(),
                "Support email".to_string(),
                "API access basique".to_string(),
            ],
        },
        "pro" => PlanInfo {
            id: "pro".to_string(),
            name: "Pro".to_string(),
            price_monthly: 29.99,
            max_projects: 50,
            max_tasks: 5000,
            features: vec![
                "50 projets max".to_string(),
                "5000 tâches max".to_string(),
                "Support prioritaire".to_string(),
                "API access complet".to_string(),
                "Exports avancés".to_string(),
            ],
        },
        "enterprise" => PlanInfo {
            id: "enterprise".to_string(),
            name: "Enterprise".to_string(),
            price_monthly: 99.99,
            max_projects: -1,
            max_tasks: -1,
            features: vec![
                "Projets illimités".to_string(),
                "Tâches illimitées".to_string(),
                "Support 24/7 dédié".to_string(),
                "API access complet".to_string(),
                "SLA 99.9% garanti".to_string(),
                "Onboarding personnalisé".to_string(),
            ],
        },
        _ => PlanInfo {
            id: "free".to_string(),
            name: "Free".to_string(),
            price_monthly: 0.0,
            max_projects: 3,
            max_tasks: 100,
            features: vec![
                "3 projets max".to_string(),
                "100 tâches max".to_string(),
                "Support communauté".to_string(),
            ],
        },
    }
}

pub fn all_plans() -> Vec<PlanInfo> {
    vec![
        get_plan_info("free"),
        get_plan_info("starter"),
        get_plan_info("pro"),
        get_plan_info("enterprise"),
    ]
}