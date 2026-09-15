use crate::enums::CertificationLevel;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorProfile {
    pub id: i32,
    pub user_id: i32,
    pub introduction: Option<String>,
    pub bio: Option<String>,
    pub rating: Decimal,
    pub certification_level: CertificationLevel,
    pub service_areas: Option<serde_json::Value>,
    pub available_slots: Option<serde_json::Value>,
    pub style_vector_id: Option<String>,
    pub portfolio_url: Option<String>,
    pub total_services: i32,
    pub total_appointments: i32,
    pub total_income: Decimal,
    pub avg_rating: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
