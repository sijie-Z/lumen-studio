use crate::state::AppState;
use axum::{extract::FromRequestParts, http::request::Parts};
use common::AppError;
use db::entities::user as user_entity;
use sea_orm::EntityTrait;
use services::auth_service::Claims;

pub struct AuthUser(pub Claims);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = common::AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(common::AppError::Unauthorized)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(common::AppError::Unauthorized)?;
        let claims = state.auth.verify_token(token)?;
        let user = user_entity::Entity::find_by_id(claims.sub)
            .one(&state.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or(AppError::Unauthorized)?;
        if user.status != "active" {
            return Err(AppError::Forbidden("user is not active".into()));
        }
        Ok(AuthUser(claims))
    }
}
