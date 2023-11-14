use tokio::sync::RwLock;

pub struct AppState {
  pub app_name: String,
}

impl AppState {
  #[allow(clippy::too_many_arguments)]
  pub fn build(app_name: String) -> AppState {
    AppState { app_name }
  }
}
