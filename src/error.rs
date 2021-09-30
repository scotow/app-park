use thiserror::Error;
use axum::response::IntoResponse;
use axum::body::Body;
use axum::http::{Response, StatusCode};

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot find app")]
    InvalidApp,
    #[error("invalid host header")]
    HostHeader,
    #[error("app binary not found")]
    AppBinary,
    #[error("app metadata not found")]
    AppMetadata,
    #[error("apps storage not found")]
    AppsStorage,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        use Error::*;
        match self {
            InvalidApp => StatusCode::NOT_FOUND,
            HostHeader => StatusCode::BAD_REQUEST,
            AppBinary => StatusCode::INTERNAL_SERVER_ERROR,
            AppMetadata => StatusCode::INTERNAL_SERVER_ERROR,
            AppsStorage => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        Response::builder()
            .status(self.status_code())
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}