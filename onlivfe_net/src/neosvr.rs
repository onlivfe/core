use neos::{
	api_client::{ApiClient, AuthenticatedNeos, UnauthenticatedNeos},
	query,
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
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
			.map_err(|_| "Internal error, API client creation failed".to_string())?;

		let reply =
			api.query(auth).await.map_err(|_| "Authentication failed".to_owned())?;
		let auth = query::Authentication::from(&reply);
		let api = api.upgrade(auth.clone()).map_err(|_| {
			"Internal error, authenticated API client's creation failed".to_owned()
		})?;
		std::mem::swap(&mut *lock, &mut Some(api));
		Ok(auth)
	}
}
