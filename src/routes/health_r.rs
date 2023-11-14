use actix_web::{get, web, HttpResponse};
use log::warn;

use crate::common::response::ApiError;

// Simple ping/pong health endpoint
#[utoipa::path(
    path = "/health/ping",
    tag = "Health",
    responses(
        (status = 200
         , description = "It should respond with \"pong\""
         , body = String),
    ),
)]
#[get("/ping")]
pub async fn ping_pong() -> Result<HttpResponse, ApiError> {
  warn!("Ping response");
  Ok(HttpResponse::Ok().body("pong"))
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  cfg.service(web::scope("/health").service(ping_pong));
}
