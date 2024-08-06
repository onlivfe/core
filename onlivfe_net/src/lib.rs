//! Network connection handling of [onlivfe](https://onlivfe.com).
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

#[macro_use]
extern crate tracing;

use std::collections::HashMap;

use ::resonite::api_client::AuthenticatedResonite;
use chilloutvr::api_client::AuthenticatedCVR;
use onlivfe::{
	Authentication,
	Instance,
	InstanceId,
	LoginCredentials,
	LoginError,
	PlatformAccount,
	PlatformAccountId,
	PlatformDataAndMetadata,
	PlatformFriend,
	PlatformType,
};
use time::OffsetDateTime;
use tokio::sync::RwLock;
use vrchat::VRChatClientState;

mod cvr;
mod resonite;
mod vrchat;

/// An unified API client interface for the different platforms
pub struct OnlivfeApiClient {
	user_agent: String,
	/// The VRChat API client
	vrc: RwLock<HashMap<vrc::id::User, VRChatClientState>>,
	/// The ChilloutVR API client
	cvr: RwLock<HashMap<chilloutvr::id::User, AuthenticatedCVR>>,
	/// The Resonite API client
	resonite: RwLock<HashMap<resonite::id::User, AuthenticatedResonite>>,
}

impl std::fmt::Debug for OnlivfeApiClient {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("OnlivfeApiClient")
			.field("user_agent", &self.user_agent)
			.finish()
	}
}

impl OnlivfeApiClient {
	/// Creates a new API client
	#[must_use]
	pub fn new(user_agent: String) -> Self {
		Self {
			vrc: RwLock::default(),
			cvr: RwLock::default(),
			resonite: RwLock::default(),
			user_agent,
		}
	}

	/// Gets the fully authenticated user ID's from the clients for a platform
	#[instrument]
	pub async fn authenticated_clients(
		&self, platform: PlatformType,
	) -> Vec<PlatformAccountId> {
		trace!("Checking authenticated API clients of {:?}", platform);
		match platform {
			PlatformType::VRChat => self
				.vrc
				.read()
				.await
				.iter()
				.filter_map(|(id, state)| {
					if matches!(state, VRChatClientState::Authenticated(_)) {
						Some(id.clone().into())
					} else {
						None
					}
				})
				.collect(),
			PlatformType::ChilloutVR => {
				self.cvr.read().await.iter().map(|v| v.0.clone().into()).collect()
			}
			PlatformType::Resonite => {
				self.resonite.read().await.iter().map(|v| v.0.clone().into()).collect()
			}
		}
	}

	/// Logs out of a certain account, removing the authentication
	///
	/// # Errors
	///
	/// If something with the logout fails.
	/// The authentication will be removed from the local API client in any case
	/// though.
	#[instrument]
	pub async fn logout(&self, id: &PlatformAccountId) -> Result<(), String> {
		trace!("Logging out of {:?}", id);
		match id {
			PlatformAccountId::VRChat(id) => self.logout_vrchat(id).await?,
			PlatformAccountId::ChilloutVR(id) => self.logout_chilloutvr(id).await?,
			PlatformAccountId::Resonite(id) => self.logout_resonite(id).await?,
		}

		Ok(())
	}

	/// Tries to log in to a certain platform using the provided information
	///
	/// # Returns
	///
	/// The authentication that should be stored for possible future use
	///
	/// # Errors
	///
	/// If login fails, for example due to authentication error.
	#[instrument]
	pub async fn login(
		&self, auth: LoginCredentials,
	) -> Result<Authentication, LoginError> {
		Ok(match auth {
			LoginCredentials::VRChat(auth) => {
				let (user_id, auth) =
					self.login_vrchat(*auth).await.map_err(|(second_factor, err)| {
						second_factor.map_or_else(
							|| LoginError::Error(err),
							|v| LoginError::RequiresAdditionalFactor(v.into()),
						)
					})?;
				Authentication::VRChat(PlatformDataAndMetadata::new_now(
					Box::new(auth),
					user_id,
				))
			}
			LoginCredentials::ChilloutVR(auth) => {
				let (user_id, auth) = self
					.login_chilloutvr(None, *auth)
					.await
					.map_err(LoginError::Error)?;
				Authentication::ChilloutVR(PlatformDataAndMetadata::new_now(
					Box::new(auth),
					user_id,
				))
			}
			LoginCredentials::Resonite(auth) => {
				let (user_id, auth) =
					self.login_resonite(*auth).await.map_err(LoginError::Error)?;
				Authentication::Resonite(PlatformDataAndMetadata::new_now(
					Box::new(auth),
					user_id,
				))
			}
		})
	}

	/// Retrieves the friends list from a platform
	///
	/// # Errors
	///
	/// If something failed with getting the friends
	#[instrument]
	pub async fn friends(
		&self,
		// TODO: Change to enum with platform specific query configs
		get_as: &PlatformAccountId,
	) -> Result<Vec<PlatformFriend>, String> {
		match get_as {
			PlatformAccountId::VRChat(id) => Ok(
				self
					.friends_vrchat(id)
					.await?
					.into_iter()
					.map(|friend| {
						PlatformFriend::VRChat(PlatformDataAndMetadata::new_now(
							Box::new(friend),
							id.clone(),
						))
					})
					.collect(),
			),
			PlatformAccountId::ChilloutVR(id) => Ok(
				self
					.friends_chilloutvr(id)
					.await?
					.into_iter()
					.map(|friend| {
						PlatformFriend::ChilloutVR(PlatformDataAndMetadata::new_now(
							Box::new(friend),
							id.clone(),
						))
					})
					.collect(),
			),
			PlatformAccountId::Resonite(id) => Ok(
				self
					.friends_resonite(id)
					.await?
					.into_iter()
					.map(|friend| {
						PlatformFriend::Resonite(PlatformDataAndMetadata::new_now(
							Box::new(friend),
							id.clone(),
						))
					})
					.collect(),
			),
		}
	}

	/// Retrieves details about an instance from the platform
	///
	/// # Errors
	///
	/// If something failed with getting the instance
	#[instrument]
	pub async fn instance(
		&self, get_as: PlatformAccountId, instance_id: InstanceId,
	) -> Result<Instance, String> {
		match instance_id {
			InstanceId::VRChat(instance_id) => {
				let PlatformAccountId::VRChat(get_as) = get_as else {
					return Err("Auth and platform types don't match!".to_owned());
				};
				let instance = self.instance_vrchat(&get_as, instance_id).await?;
				Ok(Instance::VRChat(PlatformDataAndMetadata::new_now(instance, get_as)))
			}
			InstanceId::ChilloutVR(instance_id) => {
				let PlatformAccountId::ChilloutVR(get_as) = get_as else {
					return Err("Auth and platform types don't match!".to_owned());
				};
				let instance = self.instance_chilloutvr(&get_as, instance_id).await?;
				Ok(Instance::ChilloutVR(PlatformDataAndMetadata::new_now(
					instance, get_as,
				)))
			}
			InstanceId::Resonite(instance_id) => {
				let PlatformAccountId::Resonite(get_as) = get_as else {
					return Err("Auth and platform types don't match!".to_owned());
				};
				let instance = self.instance_resonite(&get_as, instance_id).await?;
				Ok(Instance::Resonite(PlatformDataAndMetadata::new_now(
					instance, get_as,
				)))
			}
		}
	}

	/// Retrieves details about an instance from the platform
	///
	/// # Errors
	///
	/// If something failed with getting the instance
	#[instrument]
	pub async fn platform_account(
		&self, get_as: PlatformAccountId, account_id: PlatformAccountId,
	) -> Result<PlatformAccount, String> {
		match account_id {
			PlatformAccountId::VRChat(account_id) => {
				let PlatformAccountId::VRChat(get_as) = get_as else {
					return Err("Auth and platform types don't match!".to_owned());
				};
				let account = self.user_vrchat(&get_as, account_id).await?;
				Ok(PlatformAccount::VRChat(PlatformDataAndMetadata::new_now(
					Box::new(account),
					get_as,
				)))
			}
			PlatformAccountId::ChilloutVR(account_id) => {
				let PlatformAccountId::ChilloutVR(get_as) = get_as else {
					return Err("Auth and platform types don't match!".to_owned());
				};
				let account = self.user_chilloutvr(&get_as, account_id).await?;
				Ok(PlatformAccount::ChilloutVR(PlatformDataAndMetadata::new_now(
					Box::new(account),
					get_as,
				)))
			}
			PlatformAccountId::Resonite(account_id) => {
				let PlatformAccountId::Resonite(get_as) = get_as else {
					return Err("Auth and platform types don't match!".to_owned());
				};
				let account = self.user_resonite(&get_as, account_id).await?;
				Ok(PlatformAccount::Resonite(PlatformDataAndMetadata::new_now(
					Box::new(account),
					get_as,
				)))
			}
		}
	}

	/// Used to restore authentication for example on app startup
	///
	/// # Errors
	///
	/// If an error happened with the authentication check/extension/login/etc
	#[instrument]
	pub async fn reauthenticate(
		&self, auth: Authentication,
	) -> Result<Authentication, String> {
		match auth {
			Authentication::VRChat(auth) => {
				let current_account = self
					.reauthenticate_vrchat(&auth.metadata.updated_by, *auth.data.clone())
					.await?;
				Ok(Authentication::VRChat(PlatformDataAndMetadata::new_now(
					Box::new(*auth.data),
					current_account.base.id,
				)))
			}
			Authentication::ChilloutVR(auth) => {
				let (id, new_auth) = self
					.login_chilloutvr(Some(auth.metadata.updated_by), *auth.data)
					.await?;
				Ok(Authentication::ChilloutVR(PlatformDataAndMetadata::new_now(
					Box::new(new_auth),
					id,
				)))
			}
			Authentication::Resonite(mut auth) => {
				self.reauthenticate_resonite((*auth.data).clone()).await?;
				auth.metadata.updated_at = OffsetDateTime::now_utc();
				Ok(Authentication::Resonite(auth))
			}
		}
	}
}
