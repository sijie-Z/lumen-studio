use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "payments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub appointment_id: Option<i32>,
    pub user_id: i32,
    pub amount: Decimal,
    pub method: Option<String>,
    pub status: String,
    pub payment_type: String,
    pub tx_id: Option<String>,
    pub expire_time: Option<DateTime<Utc>>,
    pub payment_channel: Option<String>,
    pub refund_amount: Option<Decimal>,
    pub refund_reason: Option<String>,
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
