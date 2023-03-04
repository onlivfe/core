//! The DB storage backend that can be used to power [onlivfe](https://onlivfe.com).
//!
//! Very WIP.

#![cfg_attr(nightly, feature(doc_auto_cfg))]
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![deny(clippy::cargo)]
#![warn(missing_docs)]
#![deny(rustdoc::invalid_html_tags)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// My project my choice, tabs are literally made for indentation, spaces not.
#![allow(clippy::tabs_in_doc_comments)]
// Not much can be done about it :/
#![allow(clippy::multiple_crate_versions)]

use onlivfe::{
	storage::OnlivfeStore,
	PlatformAccount,
	PlatformAccountId,
	PlatformAuthentication,
	Profile,
	ProfileId,
};

#[derive(Debug)]
/// A database backend storage for onlivfe
pub struct OnlivfeDatabaseStorageBackend {
	/// The main database connection pool
	#[allow(dead_code)]
	db: sqlx::SqlitePool,
}

impl OnlivfeDatabaseStorageBackend {
	/// Creates a new onlivfe core interface
	///
	/// # Errors
	///
	/// If the storage connection setup fails
	pub async fn new(db_url: &str) -> Result<Self, String> {
		use sqlx::{
			sqlite::{SqlitePool, SqlitePoolOptions},
			Executor,
		};

		let db: SqlitePool = SqlitePoolOptions::new()
			.max_connections(
				(num_cpus::get() - 1)
					.try_into()
					.map(|v| if v < 1 { 1u32 } else { v })
					.unwrap_or(4),
			)
			.min_connections(1)
			.connect(db_url)
			.await
			.map_err(|e| {
				"Failed to create a DB connection pool: ".to_string() + &e.to_string()
			})?;

		db.execute("PRAGMA foreign_keys = ON; PRAGMA journal_mode=WAL;")
			.await
			.map_err(|e| {
				"Failed to configure DB pragmas: ".to_string() + &e.to_string()
			})?;

		sqlx::migrate!()
			.run(&db)
			.await
			.map_err(|e| "Failed to migrate DB: ".to_string() + &e.to_string())?;

		Ok(Self { db })
	}
}

#[async_trait::async_trait]
impl OnlivfeStore for OnlivfeDatabaseStorageBackend {
	type Err = sqlx::Error;

	async fn account_ids(
		&self, _max: usize,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		todo!();
	}

	async fn account(
		&self, _account_id: PlatformAccountId,
	) -> Result<PlatformAccount, Self::Err> {
		todo!();
	}

	async fn account_profile_ids(
		&self, _account_id: PlatformAccountId,
	) -> Result<Vec<ProfileId>, Self::Err> {
		todo!();
	}

	async fn update_account(
		&self, _account: PlatformAccount,
	) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn profile(
		&self, _profile_id: ProfileId,
	) -> Result<Profile, Self::Err> {
		todo!();
	}

	async fn profile_account_ids(
		&self, _profile_id: ProfileId,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		todo!();
	}

	async fn update_profile(&self, _profile: Profile) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn authentications(
		&self,
	) -> Result<Vec<PlatformAuthentication>, Self::Err> {
		todo!();
	}

	async fn update_authentication(
		&self, _authentication: PlatformAuthentication,
	) -> Result<bool, Self::Err> {
		todo!();
	}
}
