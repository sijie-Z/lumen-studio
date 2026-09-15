use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withdrawal {
    pub id: i32,
    pub creator_id: i32,
    pub amount: Decimal,
    pub fee: Decimal,
    pub actual_amount: Option<Decimal>,
    pub status: String,
    pub account_info: Option<serde_json::Value>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
