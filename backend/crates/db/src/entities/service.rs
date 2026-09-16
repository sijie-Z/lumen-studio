use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "services")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub creator_id: i32,
    pub type_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub duration: Option<i32>,
    pub cover_image_url: Option<String>,
    pub location: Option<String>,
    pub tags: Option<String>,
    pub options: Option<Json>,
    pub is_active: bool,
    pub is_featured: bool,
    pub appointments_count: i32,
    pub style_vector_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::creator_profile::Entity",
        from = "Column::CreatorId",
        to = "super::creator_profile::Column::Id"
    )]
    Creator,
    #[sea_orm(
        belongs_to = "super::service_type::Entity",
        from = "Column::TypeId",
        to = "super::service_type::Column::Id"
    )]
    Type,
}

impl Related<super::creator_profile::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Creator.def()
    }
}

impl Related<super::service_type::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Type.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
