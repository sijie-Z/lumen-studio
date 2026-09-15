use crate::enums::DeliveryStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delivery {
    pub id: i32,
    pub appointment_id: i32,
    pub creator_id: i32,
    pub delivery_type: String,
    pub status: DeliveryStatus,
    pub file_count: i32,
    pub cloud_link: Option<String>,
    pub tracking_number: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
