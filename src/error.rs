use axum::extract::multipart::MultipartError;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use csv_async::Error;
use sqlx::error::DatabaseError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use time::error::Parse;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
    // implement for trait `From<sqlx::Error>`
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),

    //#[error("multipart error: {0}")]
    #[error(transparent)]
    MultipartRejectionError(#[from] MultipartError),

    //
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),

    //From<FromUtf8Error>
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    //From<csv_async::Error>
    #[error(transparent)]
    CsvAsyncError(#[from] csv_async::Error),

    #[error(transparent)]
    TimeParseError(#[from] time::error::Parse),

    //ParseFloatError
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("request path not found")]
    NotFound,
    #[error("an internal server error occurred")]
    Anyhow(#[from] anyhow::Error),
}

/*
*/
impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::JsonExtractorRejection(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::MultipartRejectionError(_) => StatusCode::BAD_REQUEST,
            Self::DatabaseError(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Utf8Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FromUtf8Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CsvAsyncError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TimeParseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ParseFloatError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ParseIntError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
