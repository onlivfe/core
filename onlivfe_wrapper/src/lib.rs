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

use onlivfe::{PlatformLogin, PlatformType};

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
	dotenvy::dotenv().ok();
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()?;

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

/// The core struct that is used by apps/shells/clients to fetch data and invoke
/// actions.
///
/// The purpose of it is to hide all kinds of IO logic, and just provide a clean
/// interface. Implementation details of the API or caching shouldn't be visible
/// trough the API it provides. Other than the fact that all data might be
/// cached, so requesting fresh data should still be done.
pub struct Onlivfe<StorageBackend: onlivfe::storage::OnlivfeStore> {
	/// The local cache storage of data that is used before network requests
	pub store: StorageBackend,
	api: onlivfe_net::OnlivfeApiClient,
}

impl<StorageBackend: onlivfe::storage::OnlivfeStore> Onlivfe<StorageBackend> {
	/// Creates a new onlivfe client
	///
	/// # Errors
	///
	/// If there were issues initializing API clients due to an invalid user agent
	pub fn new(store: StorageBackend) -> Result<Self, String> {
		Ok(Self {
			store,
			api: onlivfe_net::OnlivfeApiClient::new(USER_AGENT.to_owned()),
		})
	}

	/// Checks or extends authentication, adding it into use,
	/// returning an error if it's invalid.
	///
	/// # Errors
	///
	/// If the request failed or there's no valid authentication
	pub async fn check_auth(&self, platform: PlatformType) -> Result<(), String> {
		if !self.api.has_authenticated_client(platform).await {
			return Err(
				platform.as_ref().to_owned()
					+ " does not have any authentication currently",
			);
		}
		//self.api.reauthenticate(auth).await

		Ok(())
	}

	/// Logs in to a platform
	///
	/// # Errors
	///
	/// If something failed with the login
	pub async fn login(&self, login: PlatformLogin) -> Result<(), String> {
		let platform = PlatformType::from(&login);
		if self.api.has_authenticated_client(platform).await {
			return Err(
				platform.as_ref().to_owned()
					+ " does not have any authentication currently",
			);
		}

		let auth = self.api.login(login).await?;

		if let Err(e) = self.store.update_authentication(auth).await {
			return Err(e.to_string());
		}

		Ok(())
	}

	/// Removes authentication from a platform
	///
	/// # Errors
	///
	/// If something failed with logging out
	pub async fn logout(&self, platform: PlatformType) -> Result<(), String> {
		self.api.logout(platform).await
	}
}
