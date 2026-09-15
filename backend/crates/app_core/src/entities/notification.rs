use crate::enums::{NotificationPriority, NotificationType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i32,
    pub user_id: i32,
    pub r#type: NotificationType,
    pub title: String,
    pub content: Option<String>,
    pub priority: NotificationPriority,
    pub action_url: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}
