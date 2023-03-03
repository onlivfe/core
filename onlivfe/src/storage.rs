//! The storage interface that core will use

use crate::model::{
	PlatformAccount,
	PlatformAccountId,
	PlatformAuthentication,
	Profile,
	ProfileId,
};

#[async_trait::async_trait]
/// Storage backend for onlivfe
pub trait OnlivfeStore: Send + Sync + std::fmt::Debug {
	/// The error type for operations using this storage backend
	type Err: std::error::Error;

	/// Retrieves a list of account ids
	async fn account_ids(
		&self, max: usize,
	) -> Result<Vec<PlatformAccountId>, Self::Err>;
	/// Retrieves a list of accounts
	async fn accounts(
		&self, max: usize,
	) -> Result<Vec<PlatformAccount>, Self::Err> {
		use futures::prelude::*;

		let account_ids = self.account_ids(max).await?;

		let accounts = stream::iter(account_ids.into_iter())
			.then(|account_id| async move { self.account(account_id).await })
			.try_collect()
			.await?;

		Ok(accounts)
	}
	/// Retrieves the details for an account
	async fn account(
		&self, account_id: PlatformAccountId,
	) -> Result<PlatformAccount, Self::Err>;
	/// Retrieves the profile IDs for an account
	async fn account_profile_ids(
		&self, account_id: PlatformAccountId,
	) -> Result<Vec<ProfileId>, Self::Err>;
	/// Update or store a new platform account, returning if an existing one was
	/// updated
	async fn update_account(
		&self, account: PlatformAccount,
	) -> Result<bool, Self::Err>;
	/// Retrieves the profiles for an account
	async fn account_profiles(
		&self, account_id: PlatformAccountId,
	) -> Result<Vec<Profile>, Self::Err> {
		use futures::prelude::*;

		let profile_ids = self.account_profile_ids(account_id).await?;

		let profiles = stream::iter(profile_ids.into_iter())
			.then(|profile_id| async move { self.profile(profile_id).await })
			.try_collect()
			.await?;

		Ok(profiles)
	}

	/// Retrieves the details for a profile
	async fn profile(&self, profile_id: ProfileId) -> Result<Profile, Self::Err>;
	/// Retrieves the account IDs for a profile
	async fn profile_account_ids(
		&self, profile_id: ProfileId,
	) -> Result<Vec<PlatformAccountId>, Self::Err>;
	/// Update or store a new profile, returning if an existing one was updated
	async fn update_profile(&self, value: Profile) -> Result<bool, Self::Err>;
	/// Retrieves the accounts for a profile
	async fn profile_accounts(
		&self, profile_id: ProfileId,
	) -> Result<Vec<PlatformAccount>, Self::Err> {
		use futures::prelude::*;

		let account_ids = self.profile_account_ids(profile_id).await?;

		let accounts = stream::iter(account_ids.into_iter())
			.then(|account_id| async move { self.account(account_id).await })
			.try_collect()
			.await?;

		Ok(accounts)
	}

	/// Retrieves platform authentications
	async fn authentications(
		&self,
	) -> Result<Vec<PlatformAuthentication>, Self::Err>;
	/// Update or store a platform's authentication, returning if an existing one
	/// was updated
	async fn update_authentication(
		&self, auth: PlatformAuthentication,
	) -> Result<bool, Self::Err>;
}
