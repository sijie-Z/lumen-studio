use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "withdrawals")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub creator_id: i32,
    pub amount: Decimal,
    pub fee: Decimal,
    pub actual_amount: Option<Decimal>,
    pub status: String,
    pub account_info: Option<Json>,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::creator_profile::Entity",
        from = "Column::CreatorId",
        to = "super::creator_profile::Column::Id"
    )]
    CreatorProfile,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::ReviewedBy",
        to = "super::user::Column::Id"
    )]
    Reviewer,
}

impl Related<super::creator_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatorProfile.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reviewer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
