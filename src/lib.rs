//! The core that powers [onlivfe](https://onlivfe.com).
//!
//! Very WIP.

mod apis;
pub mod models;

pub fn setup_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
	dotenvy::dotenv().ok();
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
}

/// The core struct that is used by apps/shells/clients to fetch data and invoke actions.
///
/// The purpose of it is to hide all kinds of IO logic, and just provide a clean interface.
/// Implementation details of the API or caching shouldn't be visible trough the API it provides.
/// Other than the fact that all data might be cached, so requesting fresh data should still be done.
pub struct Onlivfe {
	/// The main database connection pool
	db: sqlx::SqlitePool,
}

impl Onlivfe {
	pub async fn new(db_url: &str) -> Result<Onlivfe, String> {
		use sqlx::{sqlite::{SqlitePool, SqlitePoolOptions}, Executor};

		let db: SqlitePool = SqlitePoolOptions::new()
			.max_connections((num_cpus::get() - 1).try_into().map(|v| if v < 1 {1u32} else {v}).unwrap_or(4))
			.min_connections(1)
			.connect(db_url).await
			.map_err(|e| "Failed to create a DB connection pool: ".to_string() + &e.to_string())?;

		db.execute("PRAGMA foreign_keys = ON; PRAGMA journal_mode=WAL;").await
			.map_err(|e| "Failed to configure DB pragmas: ".to_string() + &e.to_string())?;

	sqlx::migrate!()
    .run(&db)
    .await
		.map_err(|e| "Failed to migrate DB: ".to_string() + &e.to_string())?;

		Ok(Onlivfe { db })
	}
}
