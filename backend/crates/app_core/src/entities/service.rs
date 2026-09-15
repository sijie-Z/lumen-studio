use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: i32,
    pub creator_id: i32,
    pub type_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub duration: Option<i32>,
    pub cover_image_url: Option<String>,
    pub location: Option<String>,
    pub tags: Option<String>,
    pub options: Option<serde_json::Value>,
    pub is_active: bool,
    pub is_featured: bool,
    pub appointments_count: i32,
    pub style_vector_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
