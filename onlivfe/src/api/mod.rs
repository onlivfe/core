//! Core's API connection handling

use chilloutvr::{
	api_client::{ApiClient, AuthenticatedCVR},
	query::SavedLoginCredentials,
};
use neos::api_client::AuthenticatedNeos;
use tokio::sync::RwLock;
use vrc::api_client::AuthenticatedVRC;

use crate::model::{PlatformAuthentication, PlatformLogin, PlatformType};

enum VRChatClientState {
	None,
	/// Has authentication cookie saved from login but no 2FA cookie
	Authenticating((AuthenticatedVRC, vrc::query::Authentication)),
	Authenticated(AuthenticatedVRC),
}

impl VRChatClientState {
	const fn is_some(&self) -> bool { matches!(self, Self::Authenticated(_)) }
}

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
	pub fn new(user_agent: String) -> Self {
		Self {
			vrc: RwLock::new(VRChatClientState::None),
			cvr: RwLock::default(),
			neos: RwLock::default(),
			user_agent,
		}
	}

	pub async fn has_authenticated_client(&self, platform: PlatformType) -> bool {
		match platform {
			PlatformType::VRChat => self.vrc.read().await.is_some(),
			PlatformType::ChilloutVR => self.cvr.read().await.is_some(),
			PlatformType::NeosVR => self.neos.read().await.is_some(),
		}
	}

	pub async fn logout(&self, platform: PlatformType) -> Result<(), String> {
		match platform {
			// TODO: send logout message in background
			PlatformType::VRChat => {
				*(self.vrc.write().await) = VRChatClientState::None
			}
			PlatformType::ChilloutVR => *(self.cvr.write().await) = None,
			PlatformType::NeosVR => *(self.neos.write().await) = None,
		}

		Ok(())
	}

	pub async fn login(
		&self, auth: PlatformLogin,
	) -> Result<PlatformAuthentication, String> {
		Ok(match auth {
			PlatformLogin::VRChat(auth) => PlatformAuthentication::VRChat(Box::new(
				self.login_vrchat(*auth).await?,
			)),
			PlatformLogin::ChilloutVR(auth) => PlatformAuthentication::ChilloutVR(
				Box::new(self.login_chilloutvr(*auth).await?),
			),
			PlatformLogin::NeosVR(auth) => PlatformAuthentication::NeosVR(Box::new(
				self.login_neosvr(*auth).await?,
			)),
		})
	}

	async fn login_vrchat(
		&self, auth: crate::model::vrchat::LoginRequestPart,
	) -> Result<(vrc::id::User, vrc::query::Authentication), String> {
		let mut lock = self.vrc.write().await;
		let mut api = VRChatClientState::None;
		std::mem::swap(&mut *lock, &mut api);
		match auth {
			crate::model::vrchat::LoginRequestPart::LoginRequest(auth) => {
				let api = match api {
					VRChatClientState::None => vrc::api_client::UnauthenticatedVRC::new(
						self.user_agent.clone(),
						auth,
					),
					VRChatClientState::Authenticating((api, _))
					| VRChatClientState::Authenticated(api) => api.downgrade(auth),
				}
				.map_err(|_| {
					"Internal error, API client creation failed".to_string()
				})?;

				let (login_resp, token) =
					api.login().await.map_err(|_| "Authentication failed".to_owned())?;

				let auth =
					vrc::query::Authentication { second_factor_token: None, token };

				let api = api.upgrade(auth.clone()).map_err(|_| {
					"Internal error, authenticated API client's creation failed"
						.to_owned()
				})?;

				if !login_resp.requires_additional_auth.is_empty() {
					std::mem::swap(
						&mut *lock,
						&mut VRChatClientState::Authenticating((api, auth)),
					);

					return Err(
						"2FA required : ".to_string()
							+ &(login_resp
								.requires_additional_auth
								.iter()
								.map(std::convert::AsRef::as_ref)
								.collect::<Vec<&str>>()
								.join(" ")),
					);
				}

				let user: vrc::model::User =
					api.query(vrc::query::GetCurrentUser).await.map_err(|_| {
						"Couldn't get VRC user after authenticating".to_owned()
					})?;

				std::mem::swap(&mut *lock, &mut VRChatClientState::Authenticated(api));
				Ok((user.id, auth))
			}
			#[allow(clippy::manual_let_else)]
			crate::model::vrchat::LoginRequestPart::SecondFactor(second_factor) => {
				let (api, auth) = if let VRChatClientState::Authenticating(api) = api {
					api
				} else {
					return Err("Internal error, API client creation failed".to_owned());
				};

				let (status, token) = api
					.verify_second_factor(second_factor)
					.await
					.map_err(|_| "2FA verification failed".to_string())?;
				if !status.verified {
					return Err("2FA token is not valid".to_string());
				}

				let api = api.change_second_factor(token).map_err(|_| {
					"Internal error, authenticated API client's creation failed"
						.to_string()
				})?;
				let user: vrc::model::User =
					api.query(vrc::query::GetCurrentUser).await.map_err(|_| {
						"Couldn't get VRC user after authenticating".to_owned()
					})?;

				std::mem::swap(&mut *lock, &mut VRChatClientState::Authenticated(api));

				Ok((user.id, auth))
			}
		}
	}

	async fn login_chilloutvr(
		&self, auth: chilloutvr::query::LoginCredentials,
	) -> Result<
		(chilloutvr::id::User, chilloutvr::query::SavedLoginCredentials),
		String,
	> {
		let mut lock = self.cvr.write().await;
		let mut api = None;
		std::mem::swap(&mut *lock, &mut api);
		let api = api
			.map_or_else(
				|| {
					chilloutvr::api_client::UnauthenticatedCVR::new(
						self.user_agent.clone(),
					)
				},
				chilloutvr::api_client::AuthenticatedCVR::downgrade,
			)
			.map_err(|_| "Internal error, API client creation failed".to_string())?;

		let user_auth = api
			.query(auth)
			.await
			.map_err(|_| "Authentication failed".to_owned())?
			.data;
		let (id, creds) =
			(user_auth.user_id.clone(), SavedLoginCredentials::from(user_auth));
		let api = api.upgrade(creds.clone()).map_err(|_| {
			"Internal error, authenticated API client's creation failed".to_owned()
		})?;

		std::mem::swap(&mut *lock, &mut Some(api));
		Ok((id, creds))
	}

	async fn login_neosvr(
		&self, auth: neos::query::LoginCredentials,
	) -> Result<neos::query::Authentication, String> {
		let mut lock = self.neos.write().await;
		let mut api = None;
		std::mem::swap(&mut *lock, &mut api);
		let api = api
			.map_or_else(
				|| neos::api_client::UnauthenticatedNeos::new(self.user_agent.clone()),
				neos::api_client::AuthenticatedNeos::downgrade,
			)
			.map_err(|_| "Internal error, API client creation failed".to_string())?;

		let reply =
			api.query(auth).await.map_err(|_| "Authentication failed".to_owned())?;
		let auth = neos::query::Authentication::from(&reply);
		let api = api.upgrade(auth.clone()).map_err(|_| {
			"Internal error, authenticated API client's creation failed".to_owned()
		})?;
		std::mem::swap(&mut *lock, &mut Some(api));
		Ok(auth)
	}

	/*
	pub async fn reauthenticate(
		&self, auth: crate::model::PlatformAuthentication,
	) -> Result<(), String> {
		todo!();
	}
	*/
}
