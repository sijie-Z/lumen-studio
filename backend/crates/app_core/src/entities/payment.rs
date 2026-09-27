use crate::enums::{PaymentStatus, PaymentType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: i32,
    pub appointment_id: Option<i32>,
    pub user_id: i32,
    pub amount: Decimal,
    pub method: Option<String>,
    pub status: PaymentStatus,
    pub payment_type: PaymentType,
    pub tx_id: Option<String>,
    pub expire_time: Option<DateTime<Utc>>,
    pub payment_channel: Option<String>,
    pub refund_amount: Option<Decimal>,
    pub refund_reason: Option<String>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}
