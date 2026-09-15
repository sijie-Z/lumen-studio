use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub gender: String,
    pub date_of_birth: Option<String>,
    pub bio: Option<String>,
    #[sea_orm(unique)]
    pub email: Option<String>,
    #[sea_orm(unique)]
    pub phone: Option<String>,
    pub balance: Decimal,
    pub status: String,
    pub role: String,
    pub verification_status: String,
    pub verification_message: Option<String>,
    pub verification_time: Option<DateTime<Utc>>,
    pub real_name: Option<String>,
    pub id_card: Option<String>,
    pub last_login_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
