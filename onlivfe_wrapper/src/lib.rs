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

use onlivfe::{
	Authentication,
	LoginCredentials,
	PlatformAccountId,
	PlatformFriend,
	PlatformType, PlatformAccount,
};
use onlivfe_net::LoginError;
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
		dotenvy::dotenv().ok();
		tracing_subscriber::fmt()
			.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
			.try_init()?;
		trace!("Initialized tracing");
	}

	human_panic::setup_panic!(Metadata {
		name: name.into(),
		version: version.into(),
		authors: "Onlivfe contributors".into(),
		homepage: "onlivfe.com".into(),
	});

	Ok(())
}

const USER_AGENT: &str = concat!(
	"Onlivfe/",
	env!("CARGO_PKG_VERSION"),
	" (",
	env!("CARGO_PKG_REPOSITORY"),
	")",
);

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
	/// # Errors
	///
	/// If there were issues initializing API clients due to an invalid user agent
	pub fn new(store: StorageBackend) -> Result<Self, String> {
		Ok(Self {
			store: Arc::new(store),
			api: Arc::new(onlivfe_net::OnlivfeApiClient::new(USER_AGENT.to_owned())),
		})
	}

	/// Gets all the different platforms, useful for syncing unrelated codebases
	/// to the core's capabilities
	#[instrument]
	pub async fn platforms(&self) -> Vec<PlatformType> {
		PlatformType::iter().collect()
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
	/// If something failed with retrieving the latform account
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
}
