use chilloutvr::{
	api_client::{
		ApiClient,
		ApiConfiguration,
		AuthenticatedCVR,
		UnauthenticatedCVR,
	},
	id,
	model::{ExtendedInstanceDetails, Friend, UserDetails},
	query::{self, AuthType},
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
	#[instrument]
	pub(crate) async fn logout_chilloutvr(
		&self, id: &id::User,
	) -> Result<(), String> {
		if let Some(client) = self.cvr.write().await.remove(id) {
			// CVR Does not seem to have a logout endpoint
			drop(client);
			return Ok(());
		}
		warn!("Tried to log out of non-existent account {:?}", id);
		Ok(())
	}

	#[instrument]
	pub(crate) async fn instance_chilloutvr(
		&self, id: &id::User, instance_id: id::Instance,
	) -> Result<ExtendedInstanceDetails, String> {
		trace!("Fetching CVR instance {:?} as {:?}", instance_id, id);
		let rw_lock_guard = self.cvr.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "CVR API not authenticated".to_owned())?;
		let query = query::Instance { instance_id };
		let instance_resp = api
			.query(query)
			.await
			.map_err(|_| "CVR instance query failed".to_owned())?;

		Ok(instance_resp.data)
	}

	#[instrument]
	pub(crate) async fn user_chilloutvr(
		&self, get_as: &id::User, user_id: id::User,
	) -> Result<UserDetails, String> {
		trace!("Fetching CVR user {:?} as {:?}", user_id, get_as);
		let rw_lock_guard = self.cvr.read().await;
		let api = rw_lock_guard
			.get(get_as)
			.ok_or_else(|| "CVR API not authenticated".to_owned())?;
		let query = query::UserDetails { user_id };
		let user_resp =
			api.query(query).await.map_err(|_| "CVR user query failed".to_owned())?;

		Ok(user_resp.data)
	}

	#[instrument]
	pub(crate) async fn friends_chilloutvr(
		&self, id: &id::User,
	) -> Result<Vec<Friend>, String> {
		trace!("Fetching CVR friends as {:?}", id);
		let rw_lock_guard = self.cvr.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "CVR API not authenticated".to_owned())?;
		let query = query::FriendList();
		let friends_resp = api
			.query(query)
			.await
			.map_err(|_| "CVR friends query failed".to_owned())?;

		Ok(friends_resp.data.0)
	}

	#[instrument]
	pub(crate) async fn login_chilloutvr(
		&self, possible_existing: Option<id::User>,
		auth: impl Into<AuthType> + std::fmt::Debug + Send,
	) -> Result<(id::User, chilloutvr::query::SavedLoginCredentials), String> {
		let mut api_config = ApiConfiguration::new(self.user_agent.clone());
		// TODO: Add a configuration option
		api_config.mature_content_enabled = true;

		let mut rw_lock_guard = self.cvr.write().await;
		let api = possible_existing
			.and_then(|id| rw_lock_guard.remove(&id))
			.map_or_else(
				|| UnauthenticatedCVR::new(api_config),
				AuthenticatedCVR::downgrade,
			)
			.map_err(|_| {
				"Internal error, Neos API client creation failed".to_string()
			})?;

		let auth_req: AuthType = auth.into();
		let user_auth = api
			.query(auth_req)
			.await
			.map_err(|_| "CVR authentication failed".to_owned())?
			.data;
		trace!("Auth request for {:?} was successful", &user_auth.user_id);

		let (id, creds) = (
			user_auth.user_id.clone(),
			query::SavedLoginCredentials::from(user_auth),
		);
		let api = api.upgrade(creds.clone()).map_err(|_| {
			"Internal error, authenticated CVR API client's creation failed"
				.to_owned()
		})?;

		rw_lock_guard.insert(id.clone(), api);
		Ok((id, creds))
	}
}
