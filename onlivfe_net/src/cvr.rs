use chilloutvr::{
	api_client::{ApiClient, AuthenticatedCVR, UnauthenticatedCVR},
	id,
	model::{ExtendedInstanceDetails, Friend},
	query::{self, AuthType, SavedLoginCredentials},
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
	pub(crate) async fn reauthenticate_chilloutvr(
		&self, _id: id::User, _login_profile: SavedLoginCredentials,
	) -> Result<(id::User, query::SavedLoginCredentials), String> {
		let _api = self.cvr.read().await;
		Err("Not implemented!".to_string())
	}

	pub(crate) async fn instance_chilloutvr(
		&self, instance_id: id::Instance,
	) -> Result<ExtendedInstanceDetails, String> {
		let api = self.cvr.read().await;
		let api =
			api.as_ref().ok_or_else(|| "CVR API not authenticated".to_owned())?;
		let query = query::Instance { instance_id };
		let instance_resp = api
			.query(query)
			.await
			.map_err(|_| "CVR instance query failed".to_owned())?;

		Ok(instance_resp.data)
	}

	pub(crate) async fn friends_chilloutvr(&self) -> Result<Vec<Friend>, String> {
		let api = self.cvr.read().await;
		let api =
			api.as_ref().ok_or_else(|| "CVR API not authenticated".to_owned())?;
		let query = query::FriendList();
		let friends_resp = api
			.query(query)
			.await
			.map_err(|_| "CVR friends query failed".to_owned())?;

		Ok(friends_resp.data.0)
	}

	pub(crate) async fn login_chilloutvr(
		&self, auth: query::LoginCredentials,
	) -> Result<(id::User, query::SavedLoginCredentials), String> {
		let mut lock = self.cvr.write().await;
		let mut api = None;
		std::mem::swap(&mut *lock, &mut api);
		let api = api
			.map_or_else(
				|| UnauthenticatedCVR::new(self.user_agent.clone()),
				AuthenticatedCVR::downgrade,
			)
			.map_err(|_| {
				"Internal error, CVR API client creation failed".to_string()
			})?;

		let user_auth = api
			.query(AuthType::LoginProfile(auth))
			.await
			.map_err(|_| "CVR authentication failed".to_owned())?
			.data;
		let (id, creds) = (
			user_auth.user_id.clone(),
			query::SavedLoginCredentials::from(user_auth),
		);
		let api = api.upgrade(creds.clone()).map_err(|_| {
			"Internal error, authenticated CVR API client's creation failed"
				.to_owned()
		})?;

		std::mem::swap(&mut *lock, &mut Some(api));
		Ok((id, creds))
	}
}
