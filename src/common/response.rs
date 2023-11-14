use actix_web::http::{header, StatusCode};
use actix_web::{error, HttpResponse};
use chrono::{DateTime, Utc};
use log::error;
use reqwest::header::HeaderName;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::cmp::PartialEq;
use std::fmt;
use utoipa::ToSchema;
use validator::ValidationErrors;

#[derive(ToSchema, Deserialize, Serialize, Debug, Clone)]
pub struct ApiResponse<T> {
  pub content: Vec<T>,
  #[serde()]
  pub total_pages: u64,
  pub total_elements: u64,
  pub time: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
  pub fn no_content() -> ApiResponse<T> {
    ApiResponse {
      content: Vec::new(),
      total_pages: 0,
      total_elements: 0,
      time: Utc::now(),
    }
  }

  pub fn build(content: Vec<T>, total: u64, page_size: u64) -> ApiResponse<T> {
    ApiResponse {
      content,
      total_pages: total / page_size + (if total % page_size > 0 { 1 } else { 0 }),
      total_elements: total,
      time: Utc::now(),
    }
  }

  pub fn build_mono_page(content: Vec<T>) -> ApiResponse<T> {
    ApiResponse {
      total_elements: content.len() as u64,
      content,
      total_pages: 1,
      time: Utc::now(),
    }
  }
}

#[derive(Debug, derive_more::Error, PartialEq)]
pub struct ApiError {
  pub status: StatusCode,
  pub error_message: String,
  pub validation_errors: Option<ValidationErrors>,
  pub headers: Option<Vec<(HeaderName, String)>>,
}

impl ApiError {
  pub fn from_validation_errors(validation_errors: ValidationErrors) -> Self {
    ApiError {
      status: StatusCode::BAD_REQUEST,
      error_message: String::from("Invalid payload"),
      validation_errors: Some(validation_errors),
      headers: None,
    }
  }

  pub fn bad_request(err: &str) -> ApiError {
    ApiError {
      status: StatusCode::BAD_REQUEST,
      error_message: err.to_string(),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn locked(err: String, headers: Option<Vec<(HeaderName, String)>>) -> ApiError {
    ApiError {
      status: StatusCode::LOCKED,
      error_message: err,
      validation_errors: None,
      headers,
    }
  }

  pub fn conflict(err: String) -> ApiError {
    ApiError {
      status: StatusCode::CONFLICT,
      error_message: err,
      validation_errors: None,
      headers: None,
    }
  }

  pub fn not_found() -> ApiError {
    ApiError {
      status: StatusCode::NOT_FOUND,
      error_message: String::from("Resource not found"),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn no_content() -> ApiError {
    ApiError {
      status: StatusCode::NO_CONTENT,
      error_message: String::from("No content"),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn unsupported(err: String) -> ApiError {
    ApiError {
      status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
      error_message: err,
      validation_errors: None,
      headers: None,
    }
  }

  pub fn unauthorized() -> Self {
    ApiError {
      status: StatusCode::UNAUTHORIZED,
      error_message: String::from("You are not un authorized user"),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn forbidden() -> Self {
    ApiError {
      status: StatusCode::FORBIDDEN,
      error_message: String::from("You do not have right"),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn bad_credentials() -> Self {
    ApiError {
      status: StatusCode::UNAUTHORIZED,
      error_message: String::from("Bad credentials"),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn internal_server_error(err: &str) -> Self {
    ApiError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      error_message: err.to_string(),
      validation_errors: None,
      headers: None,
    }
  }

  pub fn service_unavailable(err: String) -> Self {
    ApiError {
      status: StatusCode::SERVICE_UNAVAILABLE,
      error_message: err,
      validation_errors: None,
      headers: None,
    }
  }
}

/// Example of quick macro for ApiError.
#[macro_export]
macro_rules! err_bad_request {
  ($msg:expr) => {
    Err(ApiError::bad_request(String::from($msg)))
  };
}

impl Serialize for ApiError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    // 3 is the number of fields in the struct.
    let mut state = serializer.serialize_struct("api_error", 3)?;
    state.serialize_field("status", &self.status.as_u16())?;
    state.serialize_field("error_message", &self.error_message)?;
    state.serialize_field("validation_errors", &self.validation_errors)?;
    state.end()
  }
}

impl From<ValidationErrors> for ApiError {
  fn from(errors: ValidationErrors) -> Self {
    ApiError::from_validation_errors(errors)
  }
}

/// Functional convert sqlx error to api error.
/// It's very useful and allow us to use the elvis operator with "await?" on sqlx call.
impl From<sqlx::Error> for ApiError {
  fn from(err: sqlx::Error) -> Self {
    error!("SQL error: {:?}", err);
    ApiError {
      status: StatusCode::INTERNAL_SERVER_ERROR,
      error_message: String::from("Error occurs during database access"),
      validation_errors: None,
      headers: None,
    }
  }
}

impl fmt::Display for ApiError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let err_json = serde_json::to_string(self).unwrap();
    write!(f, "{}", err_json)
  }
}

impl error::ResponseError for ApiError {
  fn status_code(&self) -> StatusCode {
    self.status
  }
  fn error_response(&self) -> HttpResponse {
    let mut builder = HttpResponse::build(self.status);
    builder.append_header((header::CONTENT_TYPE, "application/json; charset=utf-8"));
    if let Some(headers) = self.headers.clone() {
      for hv in headers.into_iter() {
        builder.append_header(hv);
      }
    }
    builder.json(self)
  }
}

impl From<reqwest::Error> for ApiError {
  fn from(err: reqwest::Error) -> Self {
    error!("Error during 3rd party call: {:?}", err);
    ApiError::internal_server_error("Unable to call internal component")
  }
}
