use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: i32,
    pub appointment_id: i32,
    pub user_id: i32,
    pub creator_id: i32,
    pub rating: rust_decimal::Decimal,
    pub content: Option<String>,
    pub images: Option<serde_json::Value>,
    pub is_anonymous: bool,
    pub photographer_reply: Option<String>,
    pub replied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
