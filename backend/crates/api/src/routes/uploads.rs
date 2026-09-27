use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{extract::Multipart, routing::post, Json, Router};
use common::{ApiResponse, AppError};
use serde::Serialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

const MAX_UPLOAD_BYTES: usize = 15 * 1024 * 1024;
const ALLOWED_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "webp"];

#[derive(Debug, Serialize)]
pub struct UploadResult {
    pub url: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/uploads", post(upload_image))
}

async fn upload_image(
    _user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<UploadResult>>, AppError> {
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into());
    fs::create_dir_all(&upload_dir)
        .await
        .map_err(AppError::from_anyhow)?;

    let mut file_name = None;
    let mut content_type = "image/jpeg".to_string();
    let mut bytes = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("invalid multipart form: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "file" {
            file_name = field.file_name().map(str::to_owned);
            if let Some(ct) = field.content_type() {
                content_type = ct.to_string();
            }

            while let Some(chunk) = field.chunk().await.map_err(AppError::from_anyhow)? {
                bytes.extend_from_slice(&chunk);
                if bytes.len() > MAX_UPLOAD_BYTES {
                    return Err(AppError::BadRequest("image exceeds 15MB limit".into()));
                }
            }
        }
    }

    if bytes.is_empty() {
        return Err(AppError::BadRequest("file is required".into()));
    }

    let original_name = file_name.unwrap_or_else(|| "image.jpg".into());
    let extension = original_name
        .rsplit('.')
        .next()
        .unwrap_or("jpg")
        .to_lowercase();
    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(AppError::BadRequest(
            "only jpg, jpeg, png, and webp images are allowed".into(),
        ));
    }

    let stored_name = format!("{}.{}", Uuid::new_v4(), extension);
    let stored_path = std::path::Path::new(&upload_dir).join(&stored_name);
    let mut file = fs::File::create(&stored_path)
        .await
        .map_err(AppError::from_anyhow)?;
    file.write_all(&bytes)
        .await
        .map_err(AppError::from_anyhow)?;

    Ok(Json(ApiResponse::success(UploadResult {
        url: format!("/uploads/{stored_name}"),
        filename: original_name,
        content_type,
        size: bytes.len() as u64,
    })))
}
