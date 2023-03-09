use neos::{
	api_client::{ApiClient, AuthenticatedNeos, UnauthenticatedNeos},
	id,
	model::{Friend, SessionInfo, UserSession},
	query::{self, Authentication},
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
	pub(crate) async fn reauthenticate_neosvr(
		&self, _auth: Authentication,
	) -> Result<UserSession, String> {
		let _api = self.neos.read().await;
		Err("Not implemented!".to_string())
	}

	pub(crate) async fn instance_neosvr(
		&self, session_id: id::Session,
	) -> Result<SessionInfo, String> {
		let api = self.neos.read().await;
		let api =
			api.as_ref().ok_or_else(|| "Neos API not authenticated".to_owned())?;
		let query = query::SessionInfo { session_id };
		let session = api
			.query(query)
			.await
			.map_err(|_| "NeoR instance query failed".to_owned())?;

		Ok(session)
	}

	pub(crate) async fn friends_neosvr(&self) -> Result<Vec<Friend>, String> {
		let api = self.neos.read().await;
		let api =
			api.as_ref().ok_or_else(|| "Neos API not authenticated".to_owned())?;
		let query = query::Friends::default();
		let friends = api
			.query(query)
			.await
			.map_err(|_| "Neos friends query failed".to_owned())?;

		Ok(friends)
	}

	pub(crate) async fn login_neosvr(
		&self, auth: query::LoginCredentials,
	) -> Result<query::Authentication, String> {
		let mut lock = self.neos.write().await;
		let mut api = None;
		std::mem::swap(&mut *lock, &mut api);
		let api = api
			.map_or_else(
				|| UnauthenticatedNeos::new(self.user_agent.clone()),
				AuthenticatedNeos::downgrade,
			)
			.map_err(|_| {
				"Internal error, Neos API client creation failed".to_string()
			})?;

		let reply = api
			.query(auth)
			.await
			.map_err(|_| "Neos authentication failed".to_owned())?;
		let auth = query::Authentication::from(&reply);
		let api = api.upgrade(auth.clone()).map_err(|_| {
			"Internal error, authenticated Neos API client's creation failed"
				.to_owned()
		})?;
		std::mem::swap(&mut *lock, &mut Some(api));
		Ok(auth)
	}
}
