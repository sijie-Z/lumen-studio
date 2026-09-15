use sea_orm::DatabaseConnection;
use services::auth_service::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: AuthService,
}
