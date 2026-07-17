use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    AiProvider(String),
    AiPrompt(String),
    Internal(String),
    GrpcBind(String),
    HttpBind(String),
    Database(String),
    Certificate(String),
    Configuration(String),
    RateLimitExceeded,
    InvalidInput(String),
    NotFound(String),
    Forbidden(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::AiProvider(msg) => write!(f, "AI provider error: {}", msg),
            AppError::AiPrompt(msg) => write!(f, "AI prompt error: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::GrpcBind(msg) => write!(f, "gRPC bind error: {}", msg),
            AppError::HttpBind(msg) => write!(f, "HTTP bind error: {}", msg),
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Certificate(msg) => write!(f, "Certificate error: {}", msg),
            AppError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            AppError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
        }
    }
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Solver error: {0}")]
    Solver(String),
    
    #[error("Security violation: {0}")]
    Security(String),
    
    #[error("Fleet operation failed: {0}")]
    Fleet(String),
    
    #[error("Telemetry error: {0}")]
    Telemetry(String),
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::AiProvider(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::AiPrompt(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::GrpcBind(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::HttpBind(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Database(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            AppError::Certificate(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Configuration(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::RateLimitExceeded => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
        };
        (status, error_message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_app_error_display() {
        let err = AppError::AiProvider("test error".to_string());
        assert!(err.to_string().contains("AI provider error"));
    }

    #[test]
    fn test_app_error_http_status() {
        let response = AppError::RateLimitExceeded.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_app_error_invalid_input_status() {
        let response = AppError::InvalidInput("bad input".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_app_error_database_status() {
        let response = AppError::Database("connection failed".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
