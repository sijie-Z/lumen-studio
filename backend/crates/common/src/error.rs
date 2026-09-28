use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// 应用层统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
    // 400
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Validation failed: {0}")]
    Validation(String),

    // 401
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid credentials")]
    InvalidCredentials,

    // 403
    #[error("Forbidden: {0}")]
    Forbidden(String),

    // 404
    #[error("Not found: {0}")]
    NotFound(String),

    // 409
    #[error("Conflict: {0}")]
    Conflict(String),

    // 422
    #[error("Unprocessable: {0}")]
    Unprocessable(String),

    // 429
    #[error("Rate limit exceeded")]
    RateLimited,

    // 500
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn from_anyhow<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        let message = err.to_string().to_lowercase();
        if message.contains("database is locked")
            || message.contains("database table is locked")
            || message.contains("database is busy")
            || message.contains("deadlock")
        {
            AppError::Conflict("database is busy; retry the request".into())
        } else {
            AppError::Internal(anyhow::Error::new(err))
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> u16 {
        self.status_code().as_u16()
    }
}

#[derive(Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.public_message();
        let body = ErrorBody {
            code: self.code(),
            message,
        };
        (status, Json(body)).into_response()
    }
}

impl AppError {
    fn public_message(&self) -> String {
        match self {
            AppError::Internal(error) => {
                tracing::error!(error = ?error, "internal server error");
                "Internal server error".into()
            }
            other => other.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn internal_errors_do_not_expose_their_details() {
        let error = AppError::Internal(anyhow::anyhow!("database password leaked"));
        assert_eq!(error.public_message(), "Internal server error");
    }

    #[test]
    fn database_lock_errors_map_to_conflict() {
        let error = AppError::from_anyhow(std::io::Error::new(
            std::io::ErrorKind::Other,
            "database is locked",
        ));
        assert!(matches!(error, AppError::Conflict(_)));
    }
}
