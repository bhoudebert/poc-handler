use actix_web::web;
pub mod health_r;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  // Health
  cfg.configure(health_r::init_routes);

  cfg.service(web::scope("/v1").configure(|cfg| init_routes_v1(cfg)));
}

fn init_routes_v1(cfg: &mut web::ServiceConfig) {}
