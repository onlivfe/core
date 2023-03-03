//! The core that powers [onlivfe](https://onlivfe.com).
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

mod api;
pub mod model;
pub mod storage;

/// Initializes some static global parts of the core, setting up logging &
/// loading env configs and such
///
/// # Errors
///
/// If there's an issue with setting up something
pub fn init() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>
{
	dotenvy::dotenv().ok();
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
}

/// The core struct that is used by apps/shells/clients to fetch data and invoke
/// actions.
///
/// The purpose of it is to hide all kinds of IO logic, and just provide a clean
/// interface. Implementation details of the API or caching shouldn't be visible
/// trough the API it provides. Other than the fact that all data might be
/// cached, so requesting fresh data should still be done.
pub struct Onlivfe<StorageBackend: storage::OnlivfeStore> {
	/// The local cache storage of data that is used before network requests
	pub store: StorageBackend,
	api: api::OnlivfeApiClient,
}

impl<StorageBackend: storage::OnlivfeStore> Onlivfe<StorageBackend> {
	/// Creates a new onlivfe client
	///
	/// # Errors
	///
	/// If there were issues initializing API clients due to an invalid user agent
	pub fn new(
		user_agent: String, store: StorageBackend,
	) -> Result<Self, String> {
		Ok(Self { store, api: api::OnlivfeApiClient::new(user_agent)? })
	}

	/// Checks or extends authentication, adding it into use,
	/// returning an error if it's invalid.
	///
	/// # Errors
	///
	/// If the request failed or there's no valid auth
	pub async fn check_auth(
		&self, auth: model::PlatformAuthentication,
	) -> Result<(), String> {
		self.api.reauthenticate(auth).await
	}
}
