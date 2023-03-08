use onlivfe::vrchat::LoginRequestPart;
use vrc::{
	api_client::{ApiClient, AuthenticatedVRC},
	query,
};

use crate::OnlivfeApiClient;

pub enum VRChatClientState {
	None,
	/// Has authentication cookie saved from login but no 2FA cookie
	Authenticating((AuthenticatedVRC, vrc::query::Authentication)),
	Authenticated(AuthenticatedVRC),
}

impl VRChatClientState {
	pub const fn is_some(&self) -> bool { matches!(self, Self::Authenticated(_)) }
}

impl OnlivfeApiClient {
	pub(crate) async fn login_vrchat(
		&self, auth: LoginRequestPart,
	) -> Result<(vrc::id::User, query::Authentication), String> {
		let mut lock = self.vrc.write().await;
		let mut api = VRChatClientState::None;
		std::mem::swap(&mut *lock, &mut api);
		match auth {
			LoginRequestPart::LoginRequest(auth) => {
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

				let auth = query::Authentication { second_factor_token: None, token };

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

				let user: vrc::model::CurrentAccount =
					api.query(query::GetCurrentUser).await.map_err(|_| {
						"Couldn't get VRC user after authenticating".to_owned()
					})?;

				std::mem::swap(&mut *lock, &mut VRChatClientState::Authenticated(api));
				Ok((user.base.id, auth))
			}
			#[allow(clippy::manual_let_else)]
			onlivfe::vrchat::LoginRequestPart::SecondFactor(second_factor) => {
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
				let user: vrc::model::CurrentAccount =
					api.query(query::GetCurrentUser).await.map_err(|_| {
						"Couldn't get VRC user after authenticating".to_owned()
					})?;

				std::mem::swap(&mut *lock, &mut VRChatClientState::Authenticated(api));

				Ok((user.base.id, auth))
			}
		}
	}
}
