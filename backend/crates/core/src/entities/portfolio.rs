use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: i32,
    pub creator_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub category: Option<String>,
    pub is_public: bool,
    pub likes_count: i32,
    pub style_vector_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
