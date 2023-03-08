//! The storage interface that core will use

use crate::{
	Authentication,
	Avatar,
	AvatarId,
	Instance,
	InstanceId,
	PlatformAccount,
	PlatformAccountId,
	Profile,
	ProfileId,
	World,
	WorldId,
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
	/// Update or store a new platform account,
	/// returning if an existing one was updated
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

	/// Retrieves a list of instance ids
	async fn instance_ids(
		&self, max: usize,
	) -> Result<Vec<InstanceId>, Self::Err>;
	/// Retrieves a list of instances
	async fn instances(&self, max: usize) -> Result<Vec<Instance>, Self::Err> {
		use futures::prelude::*;

		let instance_ids = self.instance_ids(max).await?;

		let instances = stream::iter(instance_ids.into_iter())
			.then(|instance_id| async move { self.instance(instance_id).await })
			.try_collect()
			.await?;

		Ok(instances)
	}
	/// Retrieves the details for an instance
	async fn instance(
		&self, instance_id: InstanceId,
	) -> Result<Instance, Self::Err>;
	/// Update or store a new instance,
	/// returning if an existing one was updated
	async fn update_instance(
		&self, instance: Instance,
	) -> Result<bool, Self::Err>;

	/// Retrieves a list of world ids
	async fn world_ids(&self, max: usize) -> Result<Vec<WorldId>, Self::Err>;
	/// Retrieves a list of worlds
	async fn worlds(&self, max: usize) -> Result<Vec<World>, Self::Err> {
		use futures::prelude::*;

		let world_ids = self.world_ids(max).await?;

		let worlds = stream::iter(world_ids.into_iter())
			.then(|world_id| async move { self.world(world_id).await })
			.try_collect()
			.await?;

		Ok(worlds)
	}
	/// Retrieves the details for an account
	async fn world(&self, world_id: WorldId) -> Result<World, Self::Err>;
	/// Update or store a new world,
	/// returning if an existing one was updated
	async fn update_world(&self, world: World) -> Result<bool, Self::Err>;

	/// Retrieves a list of avatar ids
	async fn avatar_ids(&self, max: usize) -> Result<Vec<AvatarId>, Self::Err>;
	/// Retrieves a list of avatars
	async fn avatars(&self, max: usize) -> Result<Vec<Avatar>, Self::Err> {
		use futures::prelude::*;

		let avatar_ids = self.avatar_ids(max).await?;

		let avatars = stream::iter(avatar_ids.into_iter())
			.then(|avatar_id| async move { self.avatar(avatar_id).await })
			.try_collect()
			.await?;

		Ok(avatars)
	}
	/// Retrieves the details for an avatar
	async fn avatar(&self, avatar_id: AvatarId) -> Result<Avatar, Self::Err>;
	/// Update or store a new avatar,
	/// returning if an existing one was updated
	async fn update_avatar(&self, world: Avatar) -> Result<bool, Self::Err>;

	/// Retrieves the details for a profile
	async fn profile(&self, profile_id: ProfileId) -> Result<Profile, Self::Err>;
	/// Retrieves the account IDs for a profile
	async fn profile_account_ids(
		&self, profile_id: ProfileId,
	) -> Result<Vec<PlatformAccountId>, Self::Err>;
	/// Update or store a new profile,
	/// returning if an existing one was updated
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
	async fn authentications(&self) -> Result<Vec<Authentication>, Self::Err>;
	/// Update or store a platform's authentication, returning if an existing one
	/// was updated
	async fn update_authentication(
		&self, auth: Authentication,
	) -> Result<bool, Self::Err>;
}
