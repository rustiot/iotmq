use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
enum ApiError {
    #[error("{msg}")]
    Error { code: u16, msg: String },
    #[error("Api not found: {0}")]
    NotFound(String),
}

impl ApiError {
    fn code(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::Error { code, .. } => *code,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Error { msg, .. } => msg.clone(),
            _ => self.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::OK,
        };

        let json = json!( { "code": self.code(), "msg": self.message() });
        (status, Json(json)).into_response()
    }
}

#[derive(Debug, Serialize)]
struct ApiOk<T> {
    code: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl ApiOk<()> {
    fn ok() -> Self {
        Self { code: 0, msg: None, data: None }
    }

    fn msg<M: Into<String>>(msg: M) -> Self {
        Self { code: 0, msg: Some(msg.into()), data: None }
    }
}

impl<T> ApiOk<T> {
    fn data(data: T) -> Self {
        Self { code: 0, msg: None, data: Some(data) }
    }
}

impl<T: Serialize> IntoResponse for ApiOk<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route("/version", get(version)).fallback(not_found)
}

// 404 Not Found
async fn not_found(uri: OriginalUri) -> impl IntoResponse {
    ApiError::NotFound(uri.to_string())
}

async fn version() -> impl IntoResponse {
    ApiOk::data(env!("CARGO_PKG_VERSION"))
}
