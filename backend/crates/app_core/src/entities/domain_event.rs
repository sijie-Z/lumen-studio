use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: i64,
    pub aggregate_type: String,
    pub aggregate_id: i32,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub version: i32,
    pub created_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}
