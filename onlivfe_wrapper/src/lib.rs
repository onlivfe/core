//! The main interface of [onlivfe](https://onlivfe.com) that apps use.
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

#[macro_use]
extern crate tracing;

use std::sync::Arc;

use human_panic::Metadata;
pub use onlivfe;
use onlivfe::{
	Authentication,
	Instance,
	InstanceId,
	LoginCredentials,
	LoginError,
	PlatformAccount,
	PlatformAccountId,
	PlatformFriend,
	PlatformType,
	Profile,
	ProfileId,
};
// TODO: Make re-exports needless
pub use onlivfe_cache_store;
pub use onlivfe_net;
use strum::IntoEnumIterator;

/// Initializes some static global parts of the core, setting up logging &
/// loading env configs and such
///
/// # Errors
///
/// If there's an issue with setting up something
pub fn init(
	name: impl Into<std::borrow::Cow<'static, str>>,
	version: impl Into<std::borrow::Cow<'static, str>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
	let name = name.into();
	let version = version.into();
	human_panic::setup_panic!(
		Metadata::new(name.clone(), version.clone())
			.authors("Onlivfe contributors")
			.homepage("onlivfe.com")
	);

	#[cfg(target_os = "android")]
	{
		android_logger::init_once(
			android_logger::Config::default()
				.with_max_level(log::LevelFilter::Trace)
				.with_tag("onlivfe"),
		);
		trace!("Initialized tracing");
	}
	#[cfg(not(target_os = "android"))]
	{
		#[cfg(debug_assertions)]
		dotenvy::dotenv()?;
		tracing_subscriber::fmt()
    	.pretty()
			.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
			.try_init()?;
		trace!("Initialized tracing");
	}

	Ok(())
}

const USER_AGENT: &str = concat!(
	"Onlivfe/",
	env!("CARGO_PKG_VERSION"),
	" (",
	env!("CARGO_PKG_REPOSITORY"),
	")",
);

/// An error that happened with reauthenticating multiple accounts
#[derive(Debug, Clone)]
pub enum ReauthError {
	/// Retrieving the accounts from storage failed
	Storage(String),
	/// Some accounts failed to authenticate
	FailedToAuthenticate(Vec<(String, PlatformAccountId)>)
}

#[derive(Debug)]
/// The core struct that is used by apps/shells/clients to fetch data and invoke
/// actions.
///
/// The purpose of it is to hide all kinds of IO logic, and just provide a clean
/// interface. Implementation details of the API or caching shouldn't be visible
/// trough the API it provides. Other than the fact that all data might be
/// cached, so requesting fresh data should still be done.
pub struct Onlivfe<StorageBackend: onlivfe::storage::OnlivfeStore> {
	/// The local cache storage of data that is used before network requests
	store: Arc<StorageBackend>,
	/// The unified API client
	api: Arc<onlivfe_net::OnlivfeApiClient>,
}

impl<StorageBackend: onlivfe::storage::OnlivfeStore> Onlivfe<StorageBackend> {
	/// Creates a new onlivfe client
	/// 
	/// Remember to call `re_authenticate` after the creation to setup the API client properly!
	///
	/// # Errors
	///
	/// If there were issues initializing API clients due to an invalid user agent
	pub fn new(store: StorageBackend) -> Result<Self, String> {
		Ok(Self {
			store: Arc::new(store),
			api: Arc::new(onlivfe_net::OnlivfeApiClient::new(USER_AGENT.to_owned())),
		})
	}

	/// Gets currently authenticated API user IDs.
	///
	/// # Errors
	///
	/// If the request failed or there's no valid authentication
	pub async fn authenticated_accounts(
		&self,
	) -> Result<Vec<PlatformAccountId>, String> {
		let mut ids = vec![];
		for platform in PlatformType::iter() {
			ids.append(&mut self.api.authenticated_clients(platform).await);
		}

		Ok(ids)
	}

	/// Re-authenticates the API based on the storage.
	/// 
	/// Returns re-authenticated IDs
	///
	/// # Errors
	///
	/// If even getting the auths from storage failed, or if some account auths failed
	pub async fn re_authenticate(
		&self, include_already_in_api: bool
	) -> Result<Vec<PlatformAccountId>, ReauthError> {
		let mut to_authenticate = self.store.authentications().await.map_err(|e| ReauthError::Storage(e.to_string()))?;
		let mut api_ids: Vec<PlatformAccountId> = vec![];
		for platform in PlatformType::iter() {
			api_ids.append(&mut self.api.authenticated_clients(platform).await);
		}

		if !include_already_in_api {
			to_authenticate.retain(|v| !api_ids.contains(&v.id()));
		}
		let to_authenticate_ids = to_authenticate.iter().map(onlivfe::Authentication::id).collect();

		let mut errors = vec![];
		// TODO: Parallel
		for auth in to_authenticate {
			let id = auth.id();
			if let Err(e) = self.restore_login(auth).await {
				errors.push((e, id));
			}
		}

		if errors.is_empty() {
			Ok(to_authenticate_ids)
		} else {
			Err(ReauthError::FailedToAuthenticate(errors))
		}
	}

	/// Logs in to a platform
	///
	/// # Errors
	///
	/// If something failed with the login
	pub async fn login(
		&self, login: LoginCredentials,
	) -> Result<PlatformAccountId, LoginError> {
		let auth = self.api.login(login).await?;

		let id = auth.id();
		if let Err(e) = self.store.update_authentication(auth).await {
			error!("Failed to update login authentication: {e}");
			return Err(LoginError::Error(e.to_string()));
		}

		Ok(id)
	}

	/// Logs in to a platform from a previous authentication
	///
	/// # Errors
	///
	/// If the authentication session is not valid anymore or if something else
	/// failed
	pub async fn restore_login(
		&self, login: Authentication,
	) -> Result<(), String> {
		let auth = self.api.reauthenticate(login).await?;

		if let Err(e) = self.store.update_authentication(auth).await {
			error!("Failed to update restored authentication: {e}");
			return Err(e.to_string());
		}

		Ok(())
	}

	/// Removes authentication
	///
	/// # Errors
	///
	/// If something failed with logging out
	pub async fn logout(&self, id: PlatformAccountId) -> Result<(), String> {
		self.api.logout(&id).await?;

		if let Err(e) = self.store.remove_authentication(id).await {
			error!("Failed to remove stored authentication: {e}");
			return Err(e.to_string());
		}

		Ok(())
	}

	/// Gets a friend of an account
	///
	/// # Errors
	///
	/// If something failed with retrieving the friends of the platform
	pub async fn friend(
		&self, get_as: PlatformAccountId, friend_id: PlatformAccountId,
	) -> Result<PlatformFriend, String> {
		let store = self.store.clone();

		let mut friend = store.friend(friend_id.clone()).await.ok();

		let latest_updated_at = friend
			.as_ref()
			.map_or(time::OffsetDateTime::UNIX_EPOCH, |f| f.metadata().updated_at);

		// Only update our data every minute at max
		if latest_updated_at
			< time::OffsetDateTime::now_utc() - time::Duration::MINUTE
		{
			let api = self.api.clone();
			match api.friends(&get_as).await {
				Ok(friends) => {
					if let Some(friend_from_api) =
						friends.iter().find(|friend| friend.id() == friend_id)
					{
						let mut found_friend = Some(friend_from_api.clone());
						std::mem::swap(&mut found_friend, &mut friend);
					}
					if let Err(e) = store.update_friends(friends).await {
						error!("Failed to store fetched friend: {e}");
					}
				}
				Err(e) => {
					error!("Failed to fetch friends: {e}");
				}
			};
		}

		friend.ok_or_else(|| "Friend not found".to_owned())
	}

	/// Gets friends of an account
	///
	/// # Errors
	///
	/// If something failed with retrieving the friends of the platform
	pub async fn friends(
		&self, id: &PlatformAccountId,
	) -> Result<Vec<PlatformFriend>, String> {
		let store = self.store.clone();

		let mut friends =
			self.store.friends(512).await.map_err(|e| e.to_string())?;
		friends.sort_by_cached_key(|fren| fren.metadata().updated_at);

		let latest_updated_at = friends
			.last()
			.map_or(time::OffsetDateTime::UNIX_EPOCH, |f| f.metadata().updated_at);

		// Only update our data every minute at max
		if latest_updated_at
			< time::OffsetDateTime::now_utc() - time::Duration::MINUTE
		{
			let api = self.api.clone();
			match api.friends(id).await {
				Ok(friends) => {
					if let Err(e) = store.update_friends(friends).await {
						error!("Failed to store fetched friend: {e}");
						return Err(e.to_string());
					}
				}
				Err(e) => {
					error!("Failed to fetch friends: {e}");
					return Err(e);
				}
			};
		}

		Ok(friends)
	}

	/// Gets a platform account
	///
	/// # Errors
	///
	/// If something failed with retrieving the platform account
	pub async fn platform_account(
		&self, get_as: PlatformAccountId, account_id: PlatformAccountId,
	) -> Result<PlatformAccount, String> {
		let store = self.store.clone();

		let mut platform_account = store.account(account_id.clone()).await.ok();

		let latest_updated_at = platform_account
			.as_ref()
			.map_or(time::OffsetDateTime::UNIX_EPOCH, |acc| {
				acc.metadata().updated_at
			});

		// Only update our data every minute at max
		if latest_updated_at
			< time::OffsetDateTime::now_utc() - time::Duration::MINUTE
		{
			let api = self.api.clone();
			match api.platform_account(get_as, account_id).await {
				Ok(account) => {
					let mut found_account = Some(account.clone());
					std::mem::swap(&mut found_account, &mut platform_account);
					if let Err(e) = store.update_account(account).await {
						error!("Failed to store fetched platform account: {e}");
					}
				}
				Err(e) => {
					error!("Failed to fetch platform account: {e}");
				}
			};
		}

		platform_account.ok_or_else(|| "Platform account not found".to_owned())
	}

	/// Gets a platform account's profiles
	///
	/// # Errors
	///
	/// If something failed with retrieving the profiles
	pub async fn profiles(
		&self, account_id: PlatformAccountId,
	) -> Result<Vec<Profile>, String> {
		let profiles =
			self.store.account_profiles(account_id.clone()).await.map_err(|e| {
				error!(
					"Failed to get account {account_id:?} profiles from storage: {e:?}"
				);
				"Failed to retrieve account profiles".to_string()
			})?;

		Ok(profiles)
	}

	/// Updates a profile, returning a bool indicating if an existing one was
	/// updated, which gives false the meaning of the profile was now created.
	///
	/// # Errors
	///
	/// If something failed with updating the profile
	pub async fn update_profile(&self, profile: Profile) -> Result<bool, String> {
		let id = profile.sharing_id.clone();
		let was_new = self.store.update_profile(profile).await.map_err(|e| {
			error!("Failed to update profile {id:?}: {e:?}");
			"Failed to update profile".to_string()
		})?;

		Ok(was_new)
	}

	/// Gets the profile's mapped accounts
	///
	/// # Errors
	///
	/// If something failed with getting the mappings
	pub async fn profile_accounts(
		&self, profile_id: ProfileId,
	) -> Result<Vec<PlatformAccountId>, String> {
		let id = profile_id.clone();
		let account_ids =
			self.store.profile_account_ids(profile_id).await.map_err(|e| {
				error!("Failed to get profile {id:?} account mappings {e:?}");
				"Failed to get profile account mappings".to_string()
			})?;

		Ok(account_ids)
	}

	/// Updates the profile's mapped accounts
	///
	/// # Errors
	///
	/// If something failed with updating the mappings
	pub async fn update_profile_accounts(
		&self, profile_id: ProfileId, account_ids: Vec<PlatformAccountId>,
	) -> Result<(), String> {
		let id = profile_id.clone();
		self
			.store
			.update_profile_account_ids(profile_id, account_ids)
			.await
			.map_err(|e| {
				error!("Failed to update profile {id:?} account mappings {e:?}");
				"Failed to update profile account mappings".to_string()
			})?;

		Ok(())
	}

	/// Gets the accounts's mapped profiles
	///
	/// # Errors
	///
	/// If something failed with getting the mappings
	pub async fn account_profiles(
		&self, account_id: PlatformAccountId,
	) -> Result<Vec<ProfileId>, String> {
		let id = account_id.clone();
		let profile_ids =
			self.store.account_profile_ids(account_id).await.map_err(|e| {
				error!("Failed to get account {id:?} profile mappings {e:?}");
				"Failed to get account profile mappings".to_string()
			})?;

		Ok(profile_ids)
	}

	/// Updates the profile's mapped accounts
	///
	/// # Errors
	///
	/// If something failed with updating the mappings
	pub async fn update_account_profiles(
		&self, account_id: PlatformAccountId, profile_ids: Vec<ProfileId>,
	) -> Result<(), String> {
		let id = account_id.clone();
		self
			.store
			.update_account_profile_ids(account_id, profile_ids)
			.await
			.map_err(|e| {
				error!("Failed to update account {id:?} profile mappings {e:?}");
				"Failed to update account profile mappings".to_string()
			})?;

		Ok(())
	}

	/// Removes a profile fully.
	///
	/// # Errors
	///
	/// If something failed with updating the profile
	pub async fn delete_profile(
		&self, profile_id: ProfileId,
	) -> Result<(), String> {
		let id = profile_id.clone();
		self.store.delete_profile(profile_id).await.map_err(|e| {
			error!("Failed to delete profile {id:?}: {e:?}");
			"Failed to update profile".to_string()
		})?;

		Ok(())
	}

	/// Gets details about an instance
	///
	/// # Errors
	///
	/// If something failed with retrieving the details of the instance
	pub async fn instance(
		&self, get_as: PlatformAccountId, instance_id: InstanceId,
	) -> Result<Instance, String> {
		let store = self.store.clone();

		let mut instance = store.instance(instance_id.clone()).await.ok();

		let latest_updated_at =
			instance.as_ref().map_or(time::OffsetDateTime::UNIX_EPOCH, |instance| {
				instance.metadata().updated_at
			});

		// Only update our data every minute at max
		if latest_updated_at
			< time::OffsetDateTime::now_utc() - time::Duration::MINUTE
		{
			let api = self.api.clone();
			match api.instance(get_as, instance_id).await {
				Ok(instance_from_api) => {
					let mut found_instance = Some(instance_from_api.clone());
					std::mem::swap(&mut found_instance, &mut instance);
					if let Err(e) = store.update_instance(instance_from_api).await {
						error!("Failed to store fetched instance: {e}");
					}
				}
				Err(e) => {
					error!("Failed to fetch instance: {e}");
				}
			};
		}

		instance.ok_or_else(|| "Instance not found".to_owned())
	}
}
