use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "creator_profiles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub user_id: i32,
    pub introduction: Option<String>,
    pub bio: Option<String>,
    pub rating: Decimal,
    pub certification_level: String,
    pub service_areas: Option<Json>,
    pub available_slots: Option<Json>,
    pub style_vector_id: Option<String>,
    pub portfolio_url: Option<String>,
    pub total_services: i32,
    pub total_appointments: i32,
    pub total_income: Decimal,
    pub avg_rating: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
