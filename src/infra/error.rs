use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    ValidationError(validator::ValidationErrors),
    Internal(String),
}

#[derive(Serialize, ToSchema, Debug)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    pub details: Option<Vec<FieldError>>,
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            Self::ValidationError(_) => (StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_FAILED"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code) = self.status_and_code();
        let (message, details) = match self {
            AppError::ValidationError(errors) => {
                tracing::warn!("Validation errors: {:?}", errors);
                let field_errors = errors
                    .field_errors()
                    .iter()
                    .map(|(field, errs)| FieldError {
                        field: field.to_string(),
                        message: errs
                            .iter()
                            .map(|e| e.message.as_deref().unwrap_or("Invalid value").to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    })
                    .collect::<Vec<_>>();
                ("Validation failed".to_string(), Some(field_errors))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal server error: {msg}");
                ("An unexpected error occurred".to_string(), None)
            }
            _ => {
                let msg = match &self {
                    AppError::BadRequest(m)
                    | AppError::Unauthorized(m)
                    | AppError::Forbidden(m)
                    | AppError::NotFound(m)
                    | AppError::Conflict(m) => m.clone(),
                    _ => unreachable!(),
                };
                tracing::warn!("{error_code} error: {msg}");
                (msg, None)
            }
        };

        let body = Json(ErrorResponse {
            error_code: error_code.to_string(),
            message,
            details,
        });

        (status, body).into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        match &err {
            sea_orm::DbErr::Query(sea_orm::RuntimeErr::SqlxError(
                sea_orm::sqlx::Error::Database(db_err),
            )) => {
                let code = db_err.code();
                let code_ref = code.as_deref();
                if code_ref == Some("23505") {
                    return AppError::Conflict(db_err.message().to_string());
                }
                if code_ref == Some("23503") {
                    return AppError::BadRequest(format!(
                        "Relation constraint violation: {}",
                        db_err.message()
                    ));
                }
                AppError::Internal(err.to_string())
            }
            sea_orm::DbErr::RecordNotFound(msg) => AppError::NotFound(msg.clone()),
            _ => AppError::Internal(err.to_string()),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        AppError::ValidationError(errors)
    }
}
