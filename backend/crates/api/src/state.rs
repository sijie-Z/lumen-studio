use ai::ChatClient;
use sea_orm::DatabaseConnection;
use services::admin_service::AdminService;
use services::appointment_service::AppointmentService;
use services::auth_service::AuthService;
use services::creator_service::CreatorService;
use services::payment_service::PaymentService;
use services::review_service::ReviewService;
use services::service_catalog::ServiceCatalog;
use services::work_service::WorkService;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: AuthService,
    pub chat: ChatClient,
    pub works: WorkService,
    pub creators: CreatorService,
    pub services: ServiceCatalog,
    pub appointments: AppointmentService,
    pub payments: PaymentService,
    pub reviews: ReviewService,
    pub admin: AdminService,
}
