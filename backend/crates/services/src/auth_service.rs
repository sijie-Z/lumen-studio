use crate::dto::{AuthResponse, LoginInput, RegisterInput, UserDto};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use common::AppError;
use db::entities::{creator_profile as creator_entity, user as user_entity, UserModel};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthService {
    db: DatabaseConnection,
    jwt_secret: String,
    access_token_ttl: i64,
    refresh_token_ttl: i64,
}

impl AuthService {
    pub fn new(
        db: DatabaseConnection,
        jwt_secret: impl Into<String>,
        access_token_ttl: i64,
        refresh_token_ttl: i64,
    ) -> Self {
        Self {
            db,
            jwt_secret: jwt_secret.into(),
            access_token_ttl,
            refresh_token_ttl,
        }
    }

    pub async fn register(&self, input: RegisterInput) -> Result<UserDto, AppError> {
        if input.username.trim().is_empty() {
            return Err(AppError::BadRequest("username is required".into()));
        }
        if input.password.len() < 8 {
            return Err(AppError::BadRequest(
                "password must be at least 8 characters".into(),
            ));
        }

        let username = input.username.trim().to_lowercase();
        let email = input
            .email
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let phone = input
            .phone
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let existing = user_entity::Entity::find()
            .filter(user_entity::Column::Username.eq(&username))
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?;
        if existing.is_some() {
            return Err(AppError::Conflict("username already exists".into()));
        }
        if let Some(email) = email.as_deref() {
            let existing = user_entity::Entity::find()
                .filter(user_entity::Column::Email.eq(email))
                .one(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
            if existing.is_some() {
                return Err(AppError::Conflict("email already exists".into()));
            }
        }
        if let Some(phone) = phone.as_deref() {
            let existing = user_entity::Entity::find()
                .filter(user_entity::Column::Phone.eq(phone))
                .one(&self.db)
                .await
                .map_err(AppError::from_anyhow)?;
            if existing.is_some() {
                return Err(AppError::Conflict("phone already exists".into()));
            }
        }

        let password_hash = hash_password(&input.password)?;
        let now = Utc::now();
        let model = user_entity::ActiveModel {
            username: Set(username),
            password_hash: Set(password_hash),
            nickname: Set(input.nickname.unwrap_or_else(|| "anonymous".into())),
            email: Set(email),
            phone: Set(phone),
            status: Set("active".into()),
            role: Set("user".into()),
            verification_status: Set("unverified".into()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(map_user_insert_error)?;

        self.to_user_dto(model).await
    }

    pub async fn login(&self, input: LoginInput) -> Result<AuthResponse, AppError> {
        let account = input.account.trim().to_lowercase();
        let user = user_entity::Entity::find()
            .filter(
                user_entity::Column::Username
                    .eq(&account)
                    .or(user_entity::Column::Email.eq(&account)),
            )
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or(AppError::InvalidCredentials)?;

        if user.status != "active" {
            return Err(AppError::Forbidden("user is not active".into()));
        }

        verify_password(&input.password, &user.password_hash)?;

        let access_token = self.generate_token(&user, self.access_token_ttl)?;
        let refresh_token = self.generate_token(&user, self.refresh_token_ttl)?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".into(),
            expires_in: self.access_token_ttl,
            user: self.to_user_dto(user).await?,
        })
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<UserDto, AppError> {
        let user = user_entity::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        self.to_user_dto(user).await
    }

    async fn to_user_dto(&self, user: UserModel) -> Result<UserDto, AppError> {
        let has_creator_profile = creator_entity::Entity::find()
            .filter(creator_entity::Column::UserId.eq(user.id))
            .one(&self.db)
            .await
            .map_err(AppError::from_anyhow)?
            .is_some();

        Ok(user_to_dto(user, has_creator_profile))
    }

    fn generate_token(&self, user: &UserModel, ttl: i64) -> Result<String, AppError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            role: user.role.clone(),
            exp: (now + Duration::seconds(ttl)).timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("jwt encode failed: {e}")))
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("password hash failed: {e}")))
}

fn verify_password(password: &str, hash: &str) -> Result<(), AppError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid password hash: {e}")))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::InvalidCredentials)
}

fn user_to_dto(user: UserModel, has_creator_profile: bool) -> UserDto {
    let roles = roles_for(&user.role, has_creator_profile);
    UserDto {
        id: user.id,
        username: user.username,
        nickname: user.nickname,
        avatar_url: user.avatar_url,
        email: user.email,
        phone: user.phone,
        balance: user.balance,
        role: user.role,
        roles,
        status: user.status,
    }
}

fn roles_for(user_role: &str, has_creator_profile: bool) -> Vec<String> {
    let mut roles = if user_role == "admin" {
        vec!["admin".into(), "customer".into()]
    } else {
        vec!["customer".into()]
    };

    if has_creator_profile {
        roles.push("creator".into());
    }

    roles
}

fn map_user_insert_error(error: sea_orm::DbErr) -> AppError {
    let message = error.to_string();
    if message.contains("UNIQUE constraint failed") || message.contains("duplicate key value") {
        AppError::Conflict("username, email, or phone already exists".into())
    } else {
        AppError::Internal(anyhow::Error::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use db::migrations;
    use sea_orm::IntoActiveModel;

    async fn test_db() -> DatabaseConnection {
        let db = db::connect("sqlite::memory:").await.unwrap();
        migrations::run(&db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn register_login_and_verify_token() {
        let db = test_db().await;
        let service = AuthService::new(db, "test-secret", 900, 604800);

        let registered = service
            .register(RegisterInput {
                username: "alice".into(),
                password: "password123".into(),
                email: Some("alice@example.com".into()),
                phone: Some("13812341234".into()),
                nickname: Some("Alice".into()),
            })
            .await
            .unwrap();
        assert_eq!(registered.username, "alice");
        assert_eq!(registered.email.as_deref(), Some("alice@example.com"));
        assert_eq!(registered.roles, vec!["customer"]);

        let duplicate = service
            .register(RegisterInput {
                username: "alice".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: None,
            })
            .await;
        assert!(matches!(duplicate, Err(AppError::Conflict(_))));

        let duplicate_email = service
            .register(RegisterInput {
                username: "alice-email".into(),
                password: "password123".into(),
                email: Some("ALICE@example.com".into()),
                phone: None,
                nickname: None,
            })
            .await;
        assert!(matches!(duplicate_email, Err(AppError::Conflict(_))));

        let duplicate_phone = service
            .register(RegisterInput {
                username: "alice-phone".into(),
                password: "password123".into(),
                email: None,
                phone: Some("13812341234".into()),
                nickname: None,
            })
            .await;
        assert!(matches!(duplicate_phone, Err(AppError::Conflict(_))));

        let login = service
            .login(LoginInput {
                account: "alice".into(),
                password: "password123".into(),
            })
            .await
            .unwrap();
        assert_eq!(login.user.id, 1);

        let claims = service.verify_token(&login.access_token).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.role, "user");
        assert_eq!(login.user.roles, vec!["customer"]);
    }

    #[tokio::test]
    async fn role_capabilities_cover_customer_creator_and_admin() {
        let db = test_db().await;
        let service = AuthService::new(db.clone(), "test-secret", 900, 604800);

        let customer = service
            .register(RegisterInput {
                username: "cap_customer".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Customer".into()),
            })
            .await
            .unwrap();
        assert_eq!(customer.roles, vec!["customer"]);

        let creator = service
            .register(RegisterInput {
                username: "cap_creator".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Creator".into()),
            })
            .await
            .unwrap();

        let admin = service
            .register(RegisterInput {
                username: "cap_admin".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: Some("Admin".into()),
            })
            .await
            .unwrap();

        let admin_model = user_entity::Entity::find_by_id(admin.id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        let mut admin_model = admin_model.into_active_model();
        admin_model.role = Set("admin".into());
        admin_model.update(&db).await.unwrap();

        let now = Utc::now();
        creator_entity::ActiveModel {
            user_id: Set(creator.id),
            rating: Set(rust_decimal::Decimal::new(50, 1)),
            certification_level: Set("standard".into()),
            total_services: Set(0),
            total_appointments: Set(0),
            total_income: Set(rust_decimal::Decimal::ZERO),
            avg_rating: Set(rust_decimal::Decimal::ZERO),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let creator = service.get_user_by_id(creator.id).await.unwrap();
        assert_eq!(creator.roles, vec!["customer", "creator"]);

        let admin = service.get_user_by_id(admin.id).await.unwrap();
        assert_eq!(admin.roles, vec!["admin", "customer"]);
    }

    #[tokio::test]
    async fn rejects_wrong_password() {
        let db = test_db().await;
        let service = AuthService::new(db, "test-secret", 900, 604800);

        service
            .register(RegisterInput {
                username: "bob".into(),
                password: "password123".into(),
                email: None,
                phone: None,
                nickname: None,
            })
            .await
            .unwrap();

        let result = service
            .login(LoginInput {
                account: "bob".into(),
                password: "wrong-password".into(),
            })
            .await;
        assert!(matches!(result, Err(AppError::InvalidCredentials)));
    }
}
