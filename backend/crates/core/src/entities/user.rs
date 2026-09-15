use crate::enums::{RoleType, UserStatus, VerificationStatus};
use crate::value_objects::{Email, Phone};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub gender: String,
    pub date_of_birth: Option<String>,
    pub bio: Option<String>,
    pub email: Option<Email>,
    pub phone: Option<Phone>,
    pub balance: Decimal,
    pub status: UserStatus,
    pub role: RoleType,
    pub verification_status: VerificationStatus,
    pub verification_message: Option<String>,
    pub verification_time: Option<DateTime<Utc>>,
    pub real_name: Option<String>,
    pub id_card: Option<String>,
    pub last_login_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
