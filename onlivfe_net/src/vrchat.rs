use onlivfe::vrchat::LoginRequestPart;
use vrc::{
	api_client::{ApiClient, AuthenticatedVRC},
	id,
	model::Friend,
	model::Instance,
	query::{self, Logout},
};

use crate::OnlivfeApiClient;

pub enum VRChatClientState {
	/// Has authentication cookie saved from login but no 2FA cookie
	Authenticating((AuthenticatedVRC, vrc::query::Authentication)),
	Authenticated(AuthenticatedVRC),
}

impl VRChatClientState {
	pub(crate) async fn logout(&self, _id: &id::User) -> Result<(), String> {
		match self {
			Self::Authenticating(client) => {
				client
					.0
					.query(Logout)
					.await
					.map_err(|_| "Logout failed".to_string())?;
			}
			Self::Authenticated(client) => {
				client.query(Logout).await.map_err(|_| "Logout failed".to_string())?;
			}
		}

		Ok(())
	}
}

impl OnlivfeApiClient {
	#[instrument]
	pub(crate) async fn logout_vrchat(
		&self, id: &id::User,
	) -> Result<(), String> {
		if let Some(client) = self.vrc.write().await.remove(id) {
			trace!("Logging out of {:?}", id);
			return client.logout(id).await;
		}
		warn!("Tried to log out of non-existent account {:?}", id);
		Ok(())
	}

	#[instrument]
	pub(crate) async fn reauthenticate_vrchat(
		&self, id: &id::User, auth: query::Authentication,
	) -> Result<(id::User, query::Authentication), String> {
		trace!("Reauthentcating as {:?}", id);
		let rw_lock_guard = self.vrc.read().await;
		let _api = rw_lock_guard.get(id);
		Err("Not implemented!".to_string())
	}

	#[instrument]
	pub(crate) async fn instance_vrchat(
		&self, id: &id::User, instance_id: id::WorldInstance,
	) -> Result<Instance, String> {
		trace!("Fetching instance {:?} as {:?}", instance_id, id);
		let rw_lock_guard = self.vrc.read().await;
		let api = rw_lock_guard.get(id);
		match api {
			Some(VRChatClientState::Authenticated(api)) => {
				let query = query::Instance { id: instance_id };
				let instance = api
					.query(query)
					.await
					.map_err(|_| "VRChat instance query failed".to_owned())?;

				Ok(instance)
			}
			_ => Err("VRChat API not authenticated".to_string()),
		}
	}

	#[instrument]
	pub(crate) async fn friends_vrchat(
		&self, id: &id::User,
	) -> Result<Vec<Friend>, String> {
		trace!("Fetching friends as {:?}", id);
		let rw_lock_guard = self.vrc.read().await;
		let api = rw_lock_guard.get(id);
		match api {
			Some(VRChatClientState::Authenticated(api)) => {
				let mut query = vrc::query::ListFriends::default();
				query.pagination.limit = 100;
				let friends = api
					.query(query)
					.await
					.map_err(|_| "VRChat friends query failed".to_owned())?;
				Ok(friends)
			}
			_ => Err("VRChat API not authenticated".to_string()),
		}
	}

	#[instrument]
	pub(crate) async fn login_vrchat(
		&self, auth: LoginRequestPart,
	) -> Result<(id::User, query::Authentication), (Option<id::User>, String)> {
		match auth {
			LoginRequestPart::LoginRequest(auth_req) => {
				let username = auth_req.username.clone();
				trace!("Trying to login as {:?}", &username);
				let api = vrc::api_client::UnauthenticatedVRC::new(
					self.user_agent.clone(),
					auth_req,
				)
				.map_err(|_| {
					(None, "Internal error, VRC API client creation failed".to_string())
				})?;

				let (login_resp, token) = api
					.login()
					.await
					.map_err(|_| (None, "VRC authentication failed".to_owned()))?;

				let auth = query::Authentication { second_factor_token: None, token };

				let api = api.upgrade(auth.clone()).map_err(|_| {
					(
						None,
						"Internal error, authenticated VRC API client's creation failed"
							.to_owned(),
					)
				})?;

				let user: vrc::model::CurrentAccount =
					api.query(query::GetCurrentUser).await.map_err(|_| {
						(None, "Couldn't get VRC user after authenticating".to_owned())
					})?;
				trace!("Username `{}`'s ID is {:?}", &username, &user.base.id);

				if !login_resp.requires_additional_auth.is_empty() {
					trace!("Additional auth is required for {:?}", &user.base.id);
					let mut rw_lock_guard = self.vrc.write().await;
					rw_lock_guard.insert(
						user.base.id.clone(),
						VRChatClientState::Authenticating((api, auth)),
					);
					return Err((
						Some(user.base.id),
						"VRC 2FA required : ".to_string()
							+ &(login_resp
								.requires_additional_auth
								.iter()
								.map(std::convert::AsRef::as_ref)
								.collect::<Vec<&str>>()
								.join(" ")),
					));
				}

				trace!("Auth for {:?} was successful without 2FA", &user.base.id);
				let mut rw_lock_guard = self.vrc.write().await;
				rw_lock_guard
					.insert(user.base.id.clone(), VRChatClientState::Authenticated(api));
				Ok((user.base.id, auth))
			}
			onlivfe::vrchat::LoginRequestPart::SecondFactor((id, second_factor)) => {
				trace!("Continuing login for {:?}", id);
				let mut rw_lock_guard = self.vrc.write().await;
				let state = rw_lock_guard.remove(&id).ok_or_else(|| {
					(None, "VRC authentication not in progress for user".to_owned())
				})?;
				let VRChatClientState::Authenticating(api_state) = state else {
					return Err((
						Some(id),
						"Internal error, VRC API client creation failed".to_owned(),
					));
				};

				let (api, mut auth) = api_state;

				let (status, token) =
					api.verify_second_factor(second_factor).await.map_err(|_| {
						(Some(id.clone()), "VRC 2FA verification failed".to_string())
					})?;
				trace!("2FA for {:?} got status {:?}", &id, &status);
				if !status.verified {
					return Err((Some(id), "VRC 2FA token is not valid".to_string()));
				}

				let api = api.change_second_factor(token.clone()).map_err(|_| {
					(
						Some(id.clone()),
						"Internal error, authenticated VRC API client's creation failed"
							.to_string(),
					)
				})?;
				let user: vrc::model::CurrentAccount =
					api.query(query::GetCurrentUser).await.map_err(|_| {
						(Some(id), "Couldn't get VRC user after authenticating".to_owned())
					})?;

				trace!("Auth for {:?} was successful", &user.base.id);
				auth.second_factor_token = Some(token);

				let mut rw_lock_guard = self.vrc.write().await;
				rw_lock_guard
					.insert(user.base.id.clone(), VRChatClientState::Authenticated(api));
				Ok((user.base.id, auth))
			}
		}
	}
}
