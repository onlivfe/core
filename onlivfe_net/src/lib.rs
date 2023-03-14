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

use chilloutvr::api_client::AuthenticatedCVR;
use neos::api_client::AuthenticatedNeos;
use onlivfe::{
	Authentication,
	Instance,
	InstanceId,
	LoginCredentials,
	PlatformFriend,
	PlatformType,
};
use tokio::sync::RwLock;
use vrchat::VRChatClientState;

mod cvr;
mod neosvr;
mod vrchat;

/// An unified API client interface for the different platforms
pub struct OnlivfeApiClient {
	user_agent: String,
	/// The VRChat API client
	vrc: RwLock<VRChatClientState>,
	/// The ChilloutVR API client
	cvr: RwLock<Option<AuthenticatedCVR>>,
	/// The NeosVR API client
	neos: RwLock<Option<AuthenticatedNeos>>,
}

impl OnlivfeApiClient {
	/// Creates a new API client
	pub fn new(user_agent: String) -> Self {
		Self {
			vrc: RwLock::new(VRChatClientState::None),
			cvr: RwLock::default(),
			neos: RwLock::default(),
			user_agent,
		}
	}

	/// If the unified API client has authenticated for a certain platform
	pub async fn has_authenticated_client(&self, platform: PlatformType) -> bool {
		match platform {
			PlatformType::VRChat => self.vrc.read().await.is_some(),
			PlatformType::ChilloutVR => self.cvr.read().await.is_some(),
			PlatformType::NeosVR => self.neos.read().await.is_some(),
		}
	}

	/// Logs out of a certain platform, removing it's authentication
	///
	/// # Errors
	///
	/// If something with the logout fails.
	/// The authentication will be removed from the local API client in any case
	/// though.
	pub async fn logout(&self, platform: PlatformType) -> Result<(), String> {
		match platform {
			// TODO: send logout message in background
			PlatformType::VRChat => {
				*(self.vrc.write().await) = VRChatClientState::None;
			}
			PlatformType::ChilloutVR => *(self.cvr.write().await) = None,
			PlatformType::NeosVR => *(self.neos.write().await) = None,
		}

		Ok(())
	}

	/// Tries to log in to a certain platform using the provided information
	///
	/// # Errors
	///
	/// If login fails, for example due to authentication error.
	pub async fn login(
		&self, auth: LoginCredentials,
	) -> Result<Authentication, String> {
		Ok(match auth {
			LoginCredentials::VRChat(auth) => {
				Authentication::VRChat(Box::new(self.login_vrchat(*auth).await?))
			}
			LoginCredentials::ChilloutVR(auth) => Authentication::ChilloutVR(
				Box::new(self.login_chilloutvr(*auth).await?),
			),
			LoginCredentials::NeosVR(auth) => {
				Authentication::NeosVR(Box::new(self.login_neosvr(*auth).await?))
			}
		})
	}

	/// Retrieves the friends list from a platform
	///
	/// # Errors
	///
	/// If something failed with getting the friends
	pub async fn friends(
		&self,
		// TODO: Change to enum with platform specific query configs
		platform: PlatformType,
	) -> Result<Vec<PlatformFriend>, String> {
		match platform {
			PlatformType::VRChat => {
				Ok(self.friends_vrchat().await?.into_iter().map(|fren| PlatformFriend::VRChat(fren)).collect())
			}
			PlatformType::ChilloutVR => {
				Ok(self.friends_chilloutvr().await?.into_iter().map(|fren| PlatformFriend::ChilloutVR(fren)).collect())
			}
			PlatformType::NeosVR => {
				Ok(self.friends_neosvr().await?.into_iter().map(|fren| PlatformFriend::NeosVR(fren)).collect())
			}
		}
	}

	/// Retrieves details about an instance from the platform
	///
	/// # Errors
	///
	/// If something failed with getting the instance
	pub async fn instance(
		&self, instance_id: InstanceId,
	) -> Result<Instance, String> {
		match instance_id {
			InstanceId::VRChat(instance_id) => {
				Ok(Instance::VRChat(self.instance_vrchat(instance_id).await?))
			}
			InstanceId::ChilloutVR(instance_id) => {
				Ok(Instance::ChilloutVR(self.instance_chilloutvr(instance_id).await?))
			}
			InstanceId::NeosVR(instance_id) => {
				Ok(Instance::NeosVR(self.instance_neosvr(instance_id).await?))
			}
		}
	}

	/// Used to restore authentication for example on app startup
	///
	/// # Errors
	///
	/// If an error happened with the authentication check/extension/login/etc
	pub async fn reauthenticate(
		&self, auth: Authentication,
	) -> Result<Authentication, String> {
		match auth {
			Authentication::VRChat(auth) => Ok(Authentication::VRChat(Box::new(
				self.reauthenticate_vrchat(auth.0, auth.1).await?,
			))),
			Authentication::ChilloutVR(auth) => Ok(Authentication::ChilloutVR(
				Box::new(self.reauthenticate_chilloutvr(auth.0, auth.1).await?),
			)),
			Authentication::NeosVR(auth) => Ok(Authentication::NeosVR(Box::new(
				(&self.reauthenticate_neosvr(*auth).await?).into(),
			))),
		}
	}
}
