use chilloutvr::{
	api_client::{ApiClient, AuthenticatedCVR, UnauthenticatedCVR},
	id,
	query,
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
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
			.map_err(|_| "Internal error, API client creation failed".to_string())?;

		let user_auth = api
			.query(auth)
			.await
			.map_err(|_| "Authentication failed".to_owned())?
			.data;
		let (id, creds) = (
			user_auth.user_id.clone(),
			query::SavedLoginCredentials::from(user_auth),
		);
		let api = api.upgrade(creds.clone()).map_err(|_| {
			"Internal error, authenticated API client's creation failed".to_owned()
		})?;

		std::mem::swap(&mut *lock, &mut Some(api));
		Ok((id, creds))
	}
}
