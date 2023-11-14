use actix_web::web;
pub mod health_r;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
  // Health
  cfg.configure(health_r::init_routes);

  cfg.service(web::scope("/v1").configure(init_routes_v1));
}

fn init_routes_v1(_cfg: &mut web::ServiceConfig) {}
