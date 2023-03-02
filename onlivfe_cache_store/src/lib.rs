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
	model::{PlatformAccount, PlatformAccountId, Profile, ProfileId},
	storage::OnlivfeStore,
};
use tokio::sync::RwLock;

/// An in-memory only cache storage backend for onlivfe
///
/// Built for simplicity & quick iterating, not for efficiency.
#[derive(Debug, Default)]
pub struct OnlivfeCacheStorageBackend {
	profiles: RwLock<Vec<Profile>>,
	accounts: RwLock<Vec<PlatformAccount>>,
	profiles_to_accounts: RwLock<Vec<(PlatformAccountId, ProfileId)>>,
}

impl OnlivfeCacheStorageBackend {
	#[must_use]
	/// Creates a new onlivfe cache storage backend
	pub fn new() -> Self { Self::default() }
}

#[async_trait::async_trait]
impl OnlivfeStore for OnlivfeCacheStorageBackend {
	type Err = std::io::Error;

	async fn account_ids(
		&self, max: usize,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		let accounts = self.accounts.read().await;
		let accounts: Vec<PlatformAccountId> = accounts.iter().take(max).map(onlivfe::model::PlatformAccount::id).collect();
		Ok(accounts)
	}
	async fn account(
		&self, account_id: PlatformAccountId,
	) -> Result<PlatformAccount, Self::Err> {
		let accounts = self.accounts.read().await;
		if let Some(account) = accounts.iter().find(|acc| account_id == acc.id()) {
			return Ok(account.clone());
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
	}

	async fn account_profile_ids(
		&self, account_id: PlatformAccountId,
	) -> Result<Vec<ProfileId>, Self::Err> {
		let profiles_to_accounts = self.profiles_to_accounts.read().await;
		let profile_ids: Vec<ProfileId> = profiles_to_accounts
			.iter()
			.filter(|map| account_id == map.0)
			.map(|map| map.1.clone())
			.collect();
		Ok(profile_ids)
	}

	async fn update_account(
		&self, account: PlatformAccount,
	) -> Result<bool, Self::Err> {
		let mut accounts = self.accounts.write().await;
		if let Some(acc) = accounts.iter_mut().find(|acc| account.id() == acc.id())
		{
			*acc = account;
			return Ok(true);
		}

		accounts.push(account);
		Ok(false)
	}

	async fn profile(&self, profile_id: ProfileId) -> Result<Profile, Self::Err> {
		let profiles = self.profiles.read().await;
		if let Some(profile) =
			profiles.iter().find(|profile| profile_id == profile.sharing_id)
		{
			return Ok(profile.clone());
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
	}

	async fn profile_account_ids(
		&self, profile_id: ProfileId,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		let profiles_to_accounts = self.profiles_to_accounts.read().await;
		let platform_account_ids: Vec<PlatformAccountId> = profiles_to_accounts
			.iter()
			.filter(|map| profile_id == map.1)
			.map(|map| map.0.clone())
			.collect();
		Ok(platform_account_ids)
	}

	async fn update_profile(&self, profile: Profile) -> Result<bool, Self::Err> {
		let mut profiles = self.profiles.write().await;
		if let Some(prof) =
			profiles.iter_mut().find(|prof| profile.sharing_id == prof.sharing_id)
		{
			*prof = profile;
			return Ok(true);
		}

		profiles.push(profile);
		Ok(false)
	}
}
