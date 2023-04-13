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
	Authentication,
	Avatar,
	AvatarId,
	Instance,
	InstanceId,
	PlatformAccount,
	PlatformAccountId,
	PlatformFriend,
	Profile,
	ProfileId,
	World,
	WorldId,
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

	async fn update_account_profile_ids(
		&self, _account_id: PlatformAccountId, _profile_ids: Vec<ProfileId>,
	) -> Result<(), Self::Err> {
		todo!();
	}

	async fn update_account(
		&self, _account: PlatformAccount,
	) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn friend_ids(
		&self, _max: usize,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		todo!();
	}

	async fn friend(
		&self, _friend_id: PlatformAccountId,
	) -> Result<PlatformFriend, Self::Err> {
		todo!();
	}

	async fn update_friend(
		&self, _friend: PlatformFriend,
	) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn instance_ids(
		&self, _max: usize,
	) -> Result<Vec<InstanceId>, Self::Err> {
		todo!();
	}

	async fn instance(
		&self, _instance_id: InstanceId,
	) -> Result<Instance, Self::Err> {
		todo!();
	}

	async fn update_instance(
		&self, _instance: Instance,
	) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn world_ids(&self, _max: usize) -> Result<Vec<WorldId>, Self::Err> {
		todo!();
	}

	async fn world(&self, _world_id: WorldId) -> Result<World, Self::Err> {
		todo!();
	}

	async fn update_world(&self, _world: World) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn avatar_ids(&self, _max: usize) -> Result<Vec<AvatarId>, Self::Err> {
		todo!();
	}

	async fn avatar(&self, _avatar_id: AvatarId) -> Result<Avatar, Self::Err> {
		todo!();
	}

	async fn update_avatar(&self, _world: Avatar) -> Result<bool, Self::Err> {
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

	async fn update_profile_account_ids(
		&self, _profile_id: ProfileId, _account_ids: Vec<PlatformAccountId>,
	) -> Result<(), Self::Err> {
		todo!();
	}

	async fn update_profile(&self, _profile: Profile) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn delete_profile(
		&self, _profile_id: ProfileId,
	) -> Result<(), Self::Err> {
		todo!();
	}

	async fn authentications(&self) -> Result<Vec<Authentication>, Self::Err> {
		todo!();
	}

	async fn update_authentication(
		&self, _authentication: Authentication,
	) -> Result<bool, Self::Err> {
		todo!();
	}

	async fn remove_authentication(
		&self, _id: PlatformAccountId,
	) -> Result<bool, Self::Err> {
		todo!();
	}
}
