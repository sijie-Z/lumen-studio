use crate::enums::AppointmentStatus;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: i32,
    pub user_id: i32,
    pub creator_id: i32,
    pub service_id: i32,
    pub appointment_date: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub location: Option<String>,
    pub status: AppointmentStatus,
    pub total_price: Decimal,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
