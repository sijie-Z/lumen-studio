use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub id: i32,
    pub portfolio_id: i32,
    pub image_url: String,
    pub description: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub aspect_ratio: Option<String>,
    pub style_vector_id: Option<String>,
    pub quality_score: Option<rust_decimal::Decimal>,
    pub style_tags: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
