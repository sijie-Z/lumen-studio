use sea_orm::DatabaseConnection;
use ai::ChatClient;
use services::auth_service::AuthService;
use services::work_service::WorkService;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: AuthService,
    pub chat: ChatClient,
    pub works: WorkService,
}
