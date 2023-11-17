use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use sqlx::error::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error("request path not found")]
    NotFound,
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::JsonExtractorRejection(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::DatabaseError(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::JsonExtractorRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
            ApiError::DatabaseError(database_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                database_error.to_string(),
            ),
            _ => (self.status_code(), self.to_string()),
        };
        (status, message).into_response()
    }
}

/// custom trait to allow for simple mapping of database constraints to api errors
/// taken from https://github.com/davidpdrsn/realworld-axum-sqlx/blob/main/src/http/error.rs#L193
pub trait ResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<ApiError>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError> {
        self.map_err(|e| match e.into() {
            ApiError::DatabaseError(sqlx::Error::Database(dbe))
                if dbe.constraint() == Some(name) =>
            {
                map_err(dbe)
            }
            e => e,
        })
    }
}
