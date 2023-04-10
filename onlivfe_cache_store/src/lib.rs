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

use directories::ProjectDirs;
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
use tokio::sync::RwLock;

/// An in-memory only cache storage backend for onlivfe
///
/// Built for simplicity & quick iterating, not for efficiency.
#[derive(Debug)]
pub struct OnlivfeCacheStorageBackend {
	dirs: ProjectDirs,
	profiles: RwLock<Vec<Profile>>,
	accounts: RwLock<Vec<PlatformAccount>>,
	friends: RwLock<Vec<PlatformFriend>>,
	profiles_to_accounts: RwLock<Vec<(PlatformAccountId, ProfileId)>>,
	authentications: RwLock<Vec<Authentication>>,
	instances: RwLock<Vec<Instance>>,
	worlds: RwLock<Vec<World>>,
	avatars: RwLock<Vec<Avatar>>,
}

impl OnlivfeCacheStorageBackend {
	/// Creates a new onlivfe cache storage backend
	///
	/// # Errors
	///
	/// If reading previous data from disk fails
	pub fn new(app_name: &str) -> Result<Self, String> {
		let dirs =
			ProjectDirs::from("com", "Onlivfe", app_name).ok_or_else(|| {
				"Failed to get system directory paths for storage".to_owned()
			})?;

		let authentications = dirs.config_dir().join("auth.bson");
		let authentications: Vec<Authentication> = if authentications.exists() {
			let authentications =
				std::fs::File::open(authentications).map_err(|e| e.to_string())?;
			bson::from_reader(authentications).map_err(|e| e.to_string())?
		} else {
			vec![]
		};

		let profiles = dirs.config_dir().join("profiles.bson");
		let profiles: Vec<Profile> = if profiles.exists() {
			let profiles =
				std::fs::File::open(profiles).map_err(|e| e.to_string())?;
			bson::from_reader(profiles).map_err(|e| e.to_string())?
		} else {
			vec![]
		};

		let profiles_to_accounts = dirs.config_dir().join("profiles.bson");
		let profiles_to_accounts: Vec<(PlatformAccountId, ProfileId)> =
			if profiles_to_accounts.exists() {
				let profiles_to_accounts = std::fs::File::open(profiles_to_accounts)
					.map_err(|e| e.to_string())?;
				bson::from_reader(profiles_to_accounts).map_err(|e| e.to_string())?
			} else {
				vec![]
			};

		let store = Self {
			dirs,
			accounts: RwLock::default(),
			friends: RwLock::default(),
			authentications: RwLock::new(authentications),
			profiles: RwLock::new(profiles),
			profiles_to_accounts: RwLock::new(profiles_to_accounts),
			instances: RwLock::default(),
			worlds: RwLock::default(),
			avatars: RwLock::default(),
		};

		Ok(store)
	}
}

#[async_trait::async_trait]
impl OnlivfeStore for OnlivfeCacheStorageBackend {
	type Err = std::io::Error;

	async fn account_ids(
		&self, max: usize,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		let accounts = self.accounts.read().await;
		let accounts: Vec<PlatformAccountId> =
			accounts.iter().take(max).map(PlatformAccount::id).collect();
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

	async fn friend_ids(
		&self, max: usize,
	) -> Result<Vec<PlatformAccountId>, Self::Err> {
		let friends = self.friends.read().await;
		let friend_ids: Vec<PlatformAccountId> =
			friends.iter().take(max).map(PlatformFriend::id).collect();
		Ok(friend_ids)
	}

	async fn friend(
		&self, friend_id: PlatformAccountId,
	) -> Result<PlatformFriend, Self::Err> {
		let friends = self.friends.read().await;
		if let Some(friend) = friends.iter().find(|fren| friend_id == fren.id()) {
			return Ok(friend.clone());
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
	}

	async fn update_friend(
		&self, friend: PlatformFriend,
	) -> Result<bool, Self::Err> {
		let mut friends = self.friends.write().await;
		if let Some(fren) = friends.iter_mut().find(|fren| friend.id() == fren.id())
		{
			*fren = friend;
			return Ok(true);
		}

		friends.push(friend);
		Ok(false)
	}

	async fn instance_ids(
		&self, max: usize,
	) -> Result<Vec<InstanceId>, Self::Err> {
		let instances = self.instances.read().await;
		let instance_ids: Vec<InstanceId> =
			instances.iter().take(max).map(Instance::id).collect();
		Ok(instance_ids)
	}

	async fn instance(
		&self, instance_id: InstanceId,
	) -> Result<Instance, Self::Err> {
		let instances = self.instances.read().await;
		if let Some(instance) =
			instances.iter().find(|instance| instance_id == instance.id())
		{
			return Ok(instance.clone());
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
	}

	async fn update_instance(
		&self, instance: Instance,
	) -> Result<bool, Self::Err> {
		let mut instances = self.instances.write().await;
		if let Some(acc) =
			instances.iter_mut().find(|inst| instance.id() == inst.id())
		{
			*acc = instance;
			return Ok(true);
		}

		instances.push(instance);
		Ok(false)
	}

	async fn world_ids(&self, max: usize) -> Result<Vec<WorldId>, Self::Err> {
		let worlds = self.worlds.read().await;
		let world_ids: Vec<WorldId> =
			worlds.iter().take(max).map(World::id).collect();
		Ok(world_ids)
	}

	async fn world(&self, world_id: WorldId) -> Result<World, Self::Err> {
		let worlds = self.worlds.read().await;
		if let Some(world) = worlds.iter().find(|world| world_id == world.id()) {
			return Ok(world.clone());
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
	}

	async fn update_world(&self, world: World) -> Result<bool, Self::Err> {
		let mut worlds = self.worlds.write().await;
		if let Some(acc) = worlds.iter_mut().find(|avt| world.id() == avt.id()) {
			*acc = world;
			return Ok(true);
		}

		worlds.push(world);
		Ok(false)
	}

	async fn avatar_ids(&self, max: usize) -> Result<Vec<AvatarId>, Self::Err> {
		let avatars = self.avatars.read().await;
		let avatar_ids: Vec<AvatarId> =
			avatars.iter().take(max).map(Avatar::id).collect();
		Ok(avatar_ids)
	}

	async fn avatar(&self, avatar_id: AvatarId) -> Result<Avatar, Self::Err> {
		let avatars = self.avatars.read().await;
		if let Some(avatar) = avatars.iter().find(|avatar| avatar_id == avatar.id())
		{
			return Ok(avatar.clone());
		}
		Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
	}

	async fn update_avatar(&self, avatar: Avatar) -> Result<bool, Self::Err> {
		let mut avatars = self.avatars.write().await;
		if let Some(acc) = avatars.iter_mut().find(|avt| avatar.id() == avt.id()) {
			*acc = avatar;
			return Ok(true);
		}

		avatars.push(avatar);
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

	async fn update_profile(
		&self, mut profile: Profile,
	) -> Result<bool, Self::Err> {
		let profile_id = profile.sharing_id.clone();
		let mut profiles = self.profiles.write().await;

		let mut swapped = false;

		if let Some(auth) =
			profiles.iter_mut().find(|profile| profile.sharing_id == profile_id)
		{
			std::mem::swap(&mut *auth, &mut profile);
			swapped = true;
		}

		let write_result = bson::to_vec(&*profiles).map(|bytes| {
			std::fs::write(self.dirs.config_dir().join("auth.bson"), bytes)
		});

		// Undo the operation before returning, as we want same state on disk and in
		// cache always
		let mut before_exit = || {
			if swapped {
				if let Some(prof) =
					profiles.iter_mut().find(|profile| profile.sharing_id == profile_id)
				{
					std::mem::swap(&mut *prof, &mut profile);
				}
			};
		};

		match write_result {
			Ok(Ok(_)) => {}
			Ok(Err(e)) => {
				before_exit();
				return Err(e);
			}
			Err(e) => {
				before_exit();
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
			}
		};

		if !swapped {
			profiles.push(profile);
		}

		Ok(swapped)
	}

	async fn authentications(&self) -> Result<Vec<Authentication>, Self::Err> {
		Ok(self.authentications.read().await.clone())
	}

	async fn update_authentication(
		&self, mut authentication: Authentication,
	) -> Result<bool, Self::Err> {
		let auth_id = authentication.id();
		let mut authentications = self.authentications.write().await;

		let mut swapped = false;

		if let Some(auth) =
			authentications.iter_mut().find(|auth| auth.id() == auth_id)
		{
			std::mem::swap(&mut *auth, &mut authentication);
			swapped = true;
		}

		let write_result = bson::to_vec(&*authentications).map(|bytes| {
			std::fs::write(self.dirs.config_dir().join("auth.bson"), bytes)
		});

		// Undo the operation before returning, as we want same state on disk and in
		// cache always
		let mut before_exit = || {
			if swapped {
				if let Some(auth) =
					authentications.iter_mut().find(|auth| auth.id() == auth_id)
				{
					std::mem::swap(&mut *auth, &mut authentication);
				}
			}
		};

		match write_result {
			Ok(Ok(_)) => {}
			Ok(Err(e)) => {
				before_exit();
				return Err(e);
			}
			Err(e) => {
				before_exit();
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
			}
		};

		if !swapped {
			authentications.push(authentication);
		}

		Ok(swapped)
	}

	async fn remove_authentication(
		&self, id: PlatformAccountId,
	) -> Result<bool, Self::Err> {
		let mut authentications = self.authentications.write().await;

		let removed_auth = authentications
			.iter()
			.position(|auth| auth.id() == id)
			.map(|index| authentications.swap_remove(index));

		let write_result = bson::to_vec(&*authentications).map(|bytes| {
			std::fs::write(self.dirs.config_dir().join("auth.bson"), bytes)
		});

		// Undo the operation before returning, as we want same state on disk and in
		// cache always
		let before_exit = || {
			if let Some(removed_auth) = removed_auth {
				authentications.push(removed_auth);
			}
		};

		match write_result {
			Ok(Ok(_)) => {}
			Ok(Err(e)) => {
				before_exit();
				return Err(e);
			}
			Err(e) => {
				before_exit();
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
			}
		};

		Ok(true)
	}
}
