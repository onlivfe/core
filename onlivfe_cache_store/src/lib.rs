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
// The warnings are a bit too aggressive
#![allow(clippy::significant_drop_tightening)]

#![macro_use]
extern crate tracing;

// Pain, https://github.com/rust-lang/rust/issues/43244
/// Drains items from the slice if they match the condition,
/// returns the drained items
fn drain_vec<I, F>(slice: &mut Vec<I>, cond: F) -> Vec<I>
where
	F: Fn(&I) -> bool,
{
	let mut removed = vec![];
	let mut i = 0;
	while i < slice.len() {
		if cond(&mut slice[i]) {
			removed.push(slice.remove(i));
		} else {
			i += 1;
		}
	}

	removed
}

use directories::ProjectDirs;
use onlivfe::{
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
	storage::OnlivfeStore,
};
use tokio::sync::RwLock;
use tracing::{error, trace, warn};

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

		std::fs::create_dir_all(dirs.config_dir()).map_err(|e| format!("Could not create config directory: {e}"))?;

		let authentications = dirs.config_dir().join("auth.json");
		let authentications: Vec<Authentication> = if authentications.exists() {
			let authentications =
				std::fs::File::open(authentications).map_err(|e| e.to_string())?;
			serde_json::from_reader(authentications).map_err(|e| e.to_string())?
		} else {
			vec![]
		};

		let profiles = dirs.config_dir().join("profiles.json");
		let profiles: Vec<Profile> = if profiles.exists() {
			let profiles =
				std::fs::File::open(profiles).map_err(|e| e.to_string())?;
			serde_json::from_reader(profiles).map_err(|e| e.to_string())?
		} else {
			vec![]
		};

		let profiles_to_accounts = dirs.config_dir().join("mappings.json");
		let profiles_to_accounts: Vec<(PlatformAccountId, ProfileId)> =
			if profiles_to_accounts.exists() {
				let profiles_to_accounts = std::fs::File::open(profiles_to_accounts)
					.map_err(|e| e.to_string())?;
				serde_json::from_reader(profiles_to_accounts).map_err(|e| e.to_string())?
			} else {
				vec![]
			};

		trace!("Loaded storage backed with {} authentications, {} profiles, and {} mappings", authentications.len(), profiles.len(), profiles_to_accounts.len());
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

	fn update_mappings(
		&self, mappings: &Vec<(PlatformAccountId, ProfileId)>,
	) -> Result<(), std::io::Error> {
		let write_result = serde_json::to_vec::<Vec<(PlatformAccountId, ProfileId)>>(mappings).map(|bytes| {
			trace!("Writing {} mapping bytes to disk", bytes.len());
			std::fs::write(self.dirs.config_dir().join("mappings.json"), bytes)
		});

		match write_result {
			Ok(Ok(())) => Ok(()),
			Ok(Err(e)) => Err(e),
			Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)),
		}
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

	async fn update_account_profile_ids(
		&self, account_id: PlatformAccountId, profile_ids: Vec<ProfileId>,
	) -> Result<(), Self::Err> {
		let mut profiles_to_accounts = self.profiles_to_accounts.write().await;
		let removed_profile_ids =
			drain_vec(&mut profiles_to_accounts, |(acc_id, _)| acc_id == &account_id)
				.into_iter()
				.filter(|(_, prof_id)| profile_ids.contains(prof_id))
				.map(|(_, prof_id)| prof_id)
				.collect::<Vec<ProfileId>>();
		for profile_id in profile_ids {
			profiles_to_accounts.push((account_id.clone(), profile_id));
		}

		if let Err(e) = self.update_mappings(&profiles_to_accounts) {
			for profile_id in removed_profile_ids {
				profiles_to_accounts.push((account_id.clone(), profile_id));
			}
			return Err(e);
		}

		Ok(())
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

	async fn update_profile_account_ids(
		&self, profile_id: ProfileId, account_ids: Vec<PlatformAccountId>,
	) -> Result<(), Self::Err> {
		let mut profiles_to_accounts = self.profiles_to_accounts.write().await;
		let removed_account_ids =
			drain_vec(&mut profiles_to_accounts, |(_, prof_id)| {
				prof_id == &profile_id
			})
			.into_iter()
			.filter(|(acc_id, _)| account_ids.contains(acc_id))
			.map(|(acc_id, _)| acc_id)
			.collect::<Vec<PlatformAccountId>>();
		for account_id in account_ids {
			profiles_to_accounts.push((account_id, profile_id.clone()));
		}

		if let Err(e) = self.update_mappings(&profiles_to_accounts) {
			for account_id in removed_account_ids {
				profiles_to_accounts.push((account_id, profile_id.clone()));
			}
			return Err(e);
		}

		Ok(())
	}

	async fn update_profile(
		&self, mut profile: Profile,
	) -> Result<bool, Self::Err> {
		let profile_id = profile.sharing_id.clone();
		let mut profiles = self.profiles.write().await;

		let mut swapped_profile = None;

		if let Some(auth) =
			profiles.iter_mut().find(|profile| profile.sharing_id == profile_id)
		{
			trace!("Swapping profile");
			std::mem::swap(&mut *auth, &mut profile);
			swapped_profile = Some(profile);
		} else {
			trace!("Adding profile");
			profiles.push(profile);
		}

		trace!("Going to write {} ", profiles.len());
		let write_result = serde_json::to_vec::<Vec<Profile>>(&*profiles).map(|bytes| {
			trace!("Writing {} auth bytes to disk", bytes.len());
			std::fs::write(self.dirs.config_dir().join("profiles.json"), bytes)
		});

		let swapped = swapped_profile.is_some();

		// Undo the operation before returning, as we want same state on disk and in
		// cache always
		let before_exit = || {
			if let Some(mut swapped_profile) = swapped_profile {
				trace!("Undoing profile swap");
				if let Some(prof) =
					profiles.iter_mut().find(|profile| profile.sharing_id == profile_id)
				{
					std::mem::swap(&mut *prof, &mut swapped_profile);
				} else {
					error!("This should be impossible, couldn't undo profile swap");
				}
			};
		};

		match write_result {
			Ok(Ok(())) => {}
			Ok(Err(e)) => {
				before_exit();
				return Err(e);
			}
			Err(e) => {
				before_exit();
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
			}
		};

		trace!("Fully updated profiles");

		Ok(swapped)
	}

	async fn delete_profile(
		&self, profile_id: ProfileId,
	) -> Result<(), Self::Err> {
		let mut profiles_to_accounts = self.profiles_to_accounts.write().await;
		let removed_account_ids =
			drain_vec(&mut profiles_to_accounts, |(_, prof_id)| {
				prof_id == &profile_id
			})
			.into_iter()
			.map(|(acc_id, _)| acc_id)
			.collect::<Vec<PlatformAccountId>>();

		if let Err(e) = self.update_mappings(&profiles_to_accounts) {
			for account_id in removed_account_ids {
				profiles_to_accounts.push((account_id, profile_id.clone()));
			}
			return Err(e);
		}

		Ok(())
	}

	async fn authentications(&self) -> Result<Vec<Authentication>, Self::Err> {
		Ok(self.authentications.read().await.clone())
	}

	async fn update_authentication(
		&self, mut authentication: Authentication,
	) -> Result<bool, Self::Err> {
		let auth_id = authentication.id();
		let mut authentications = self.authentications.write().await;

		let mut swapped_auth = None;

		if let Some(auth) =
			authentications.iter_mut().find(|auth| auth.id() == auth_id)
		{
			trace!("Swapping authentication");
			std::mem::swap(&mut *auth, &mut authentication);
			swapped_auth = Some(authentication);
		} else {
			trace!("Adding authentication");
			authentications.push(authentication);
		}

		trace!("Going to write {} authentications", authentications.len());
		let write_result = serde_json::to_vec::<Vec<Authentication>>(&authentications).map(|bytes| {
			trace!("Writing {} auth bytes to disk", bytes.len());
			std::fs::write(self.dirs.config_dir().join("auth.json"), bytes)
		});

		let swapped = swapped_auth.is_some();

		// Undo the operation before returning, as we want same state on disk and in
		// cache always
		let before_exit = || {
			if let Some(mut swapped_auth) = swapped_auth {
				trace!("Undoing auth swap");
				if let Some(auth) =
					authentications.iter_mut().find(|auth| auth.id() == auth_id)
				{
					std::mem::swap(&mut *auth, &mut swapped_auth);
				} else {
					error!("This should be impossible, couldn't undo auth swap");
				}
			}
		};

		match write_result {
			Ok(Ok(())) => {}
			Ok(Err(e)) => {
				before_exit();
				return Err(e);
			}
			Err(e) => {
				before_exit();
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
			}
		};

		trace!("Fully added authentication");

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

		trace!("Going to write {} authentications", authentications.len());
		let write_result = serde_json::to_vec::<Vec<Authentication>>(&*authentications).map(|bytes| {
			trace!("Writing {} auth bytes to disk", bytes.len());
			std::fs::write(self.dirs.config_dir().join("auth.json"), bytes)
		});

		// Undo the operation before returning, as we want same state on disk and in
		// cache always
		let before_exit = || {
			if let Some(removed_auth) = removed_auth {
				trace!("Undoing auth removal");
				authentications.push(removed_auth);
			}
		};

		match write_result {
			Ok(Ok(())) => {}
			Ok(Err(e)) => {
				before_exit();
				return Err(e);
			}
			Err(e) => {
				before_exit();
				return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
			}
		};

		trace!("Fully removed authentication");

		Ok(true)
	}
}
