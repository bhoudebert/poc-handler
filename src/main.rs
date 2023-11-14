extern crate magic_crypt;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use actix_web_httpauth::extractors::basic;
use anyhow::Result;
use dotenvy::dotenv;
use poc_handler::config::state::AppState;
use poc_handler::routes;
use std::env;

#[actix_web::main]
async fn main() -> Result<()> {
  dotenv().ok();

  std::env::set_var(
    "RUST_LOG",
    env::var("LOGGING_LEVELS").unwrap_or_else(|_| "error,actix_web=error".to_string()),
  );
  std::env::set_var("RUST_BACKTRACE", "1");

  env_logger::init();

  let app_name = env::var("APP_NAME").expect("MISSING APP_NAME ENV");

  HttpServer::new(move || {
    let state = AppState::build(app_name.clone());
    let cors = Cors::default()
      .allow_any_origin()
      .allow_any_header()
      .allow_any_method();
    App::new()
      .wrap(Logger::default())
      .wrap(cors)
      .app_data(Data::new(state))
      .app_data(basic::Config::default().realm("Restricted area"))
      .configure(routes::init_routes)
  })
  .bind(("0.0.0.0", 9192))?
  .run()
  .await?;

  Ok(())
}
