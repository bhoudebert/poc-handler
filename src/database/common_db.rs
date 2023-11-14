use async_trait::async_trait;
use sqlx::pool::PoolConnection;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DatabaseWrapper {
  db_pool: Arc<Pool<Postgres>>,
}

#[async_trait]
pub trait DatabaseWrapperCommands {
  fn build(pg_pool: Pool<Postgres>) -> Self;
  async fn get_conn(&self) -> Result<PoolConnection<Postgres>, sqlx::Error>;
  fn get_pool(&self) -> &Arc<Pool<Postgres>>;
}

#[async_trait]
impl DatabaseWrapperCommands for DatabaseWrapper {
  fn build(pg_pool: Pool<Postgres>) -> Self {
    Self {
      db_pool: Arc::new(pg_pool),
    }
  }

  async fn get_conn(&self) -> Result<PoolConnection<Postgres>, sqlx::Error> {
    let k1 = self.db_pool.acquire().await?;
    Ok(k1)
  }
  fn get_pool(&self) -> &Arc<Pool<Postgres>> {
    &self.db_pool
  }
}

/// Get a connection from DatabaseWrapper
/// Warning, it use a temporary because of direct use of deref.
#[macro_export]
macro_rules! get_connection {
  ($db_wrapper:expr) => {
    $db_wrapper.get_conn().await?.deref_mut()
  };
}

/// Unlike get_connection, without immediate deref
/// it can be store into variable.
#[macro_export]
macro_rules! get_var_connection {
  ($db_wrapper:expr, $conn:ident) => {
    let mut pool_conn = $db_wrapper.get_conn().await?;
    let $conn = pool_conn.deref_mut();
  };
}

#[macro_export]
macro_rules! start_transaction {
  ($db_wrapper:expr, $tx:ident) => {
    let mut $tx = $db_wrapper.get_pool().begin().await?;
  };
}

#[macro_export]
macro_rules! commit_transaction {
  ($tx:ident) => {
    $tx.commit().await?;
  };
}
