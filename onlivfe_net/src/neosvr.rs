use neos::{
	api_client::{ApiClient, AuthenticatedNeos, UnauthenticatedNeos},
	id,
	model::{Friend, SessionInfo, User},
	query::{self, Authentication, LoginCredentialsIdentifier},
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
	#[instrument]
	pub(crate) async fn logout_neos(&self, id: &id::User) -> Result<(), String> {
		if let Some(client) = self.neos.write().await.remove(id) {
			client.query(query::DestroyUserSession).await.map_err(|e| {
				error!("Logout as {:?} failed: {:?}", id, e);
				"Logout request failed, removing account anyway".to_string()
			})?;
			return Ok(());
		}
		warn!("Tried to log out of non-existent account {:?}", id);
		Ok(())
	}

	// Auth contains user ID so not passing it in here unlike other clients
	#[instrument]
	pub(crate) async fn reauthenticate_neosvr(
		&self, auth: Authentication,
	) -> Result<(), String> {
		trace!("Reauthentcating as {:?}", &auth.user_id);
		if let Some(api) = self.neos.read().await.get(&auth.user_id) {
			warn!(
				"Already had authenticated client for reauthentication as {:?}",
				auth.user_id
			);
			api.query(query::ExtendUserSession).await.map_err(|e| {
				error!(
					"Reauthentication via user session extension check as {:?} failed: {:?}",
					auth.user_id, e
				);
				"Reauthentication failed".to_owned()
			})?;
		} else {
			let id = auth.user_id.clone();
			let mut rw_lock_guard = self.neos.write().await;
			let api =
				AuthenticatedNeos::new(self.user_agent.clone(), auth).map_err(|e| {
					error!("Creating Neos API client as {id:?} failed: {e:?}");
					"Internal error, Neos API client creation failed".to_string()
				})?;

			api.query(query::ExtendUserSession).await.map_err(|e| {
				error!(
					"Reauthentication via user session extension check as {:?} failed: {:?}",
					&id, e
				);
				"Reauthentication failed".to_owned()
			})?;

			rw_lock_guard.insert(id, api);
		}

		Ok(())
	}

	#[instrument]
	pub(crate) async fn extend_auth_neosvr(
		&self, id: &id::User,
	) -> Result<(), String> {
		let rw_lock_guard = self.neos.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "Neos API not authenticated".to_owned())?;
		api.query(query::ExtendUserSession).await.map_err(|e| {
			error!("User session extension as {:?} failed: {:?}", id, e);
			"User session extension failed".to_owned()
		})?;

		Ok(())
	}

	#[instrument]
	pub(crate) async fn instance_neosvr(
		&self, id: &id::User, session_id: id::Session,
	) -> Result<SessionInfo, String> {
		trace!("Fetching instance {:?} as {:?}", session_id, id);
		let rw_lock_guard = self.neos.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "Neos API not authenticated".to_owned())?;
		let query = query::SessionInfo { session_id };
		let session = api
			.query(query)
			.await
			.map_err(|_| "Neos instance query failed".to_owned())?;

		Ok(session)
	}

	#[instrument]
	pub(crate) async fn user_neosvr(
		&self, get_as: &id::User, user_id: id::User,
	) -> Result<User, String> {
		trace!("Fetching user {:?} as {:?}", user_id, get_as);
		let rw_lock_guard = self.neos.read().await;
		let api = rw_lock_guard
			.get(get_as)
			.ok_or_else(|| "Neos API not authenticated".to_owned())?;
		let query = query::UserInfo::new(user_id);
		let user = api
			.query(query)
			.await
			.map_err(|_| "Neos user query failed".to_owned())?;

		Ok(user)
	}

	#[instrument]
	pub(crate) async fn friends_neosvr(
		&self, id: &id::User,
	) -> Result<Vec<Friend>, String> {
		trace!("Fetching friends as {:?}", id);
		let rw_lock_guard = self.neos.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "Neos API not authenticated".to_owned())?;
		let query = query::Friends::default();
		let friends = api
			.query(query)
			.await
			.map_err(|_| "Neos friends query failed".to_owned())?;

		Ok(friends)
	}

	#[instrument]
	pub(crate) async fn login_neosvr(
		&self, auth: query::LoginCredentials,
	) -> Result<(id::User, query::Authentication), String> {
		trace!("Trying to login as {:?}", auth.identifier);
		let mut rw_lock_guard = self.neos.write().await;
		let api = match &auth.identifier {
			LoginCredentialsIdentifier::OwnerID(owner_id_str) => {
				id::User::try_from(owner_id_str.clone())
					.map(|user_id| rw_lock_guard.remove(&user_id))
					.ok()
					.flatten()
			}
			_ => None,
		};
		let api = api
			.map_or_else(
				|| UnauthenticatedNeos::new(self.user_agent.clone()),
				AuthenticatedNeos::downgrade,
			)
			.map_err(|_| {
				"Internal error, Neos API client creation failed".to_string()
			})?;

		let user_session = api
			.query(auth)
			.await
			.map_err(|_| "Neos authentication failed".to_owned())?;
		trace!("Auth request for {:?} was successful", &user_session.user_id);

		let auth = query::Authentication::from(&user_session);

		let api = api.upgrade(auth.clone()).map_err(|_| {
			"Internal error, authenticated Neos API client's creation failed"
				.to_owned()
		})?;
		rw_lock_guard.insert(user_session.user_id.clone(), api);
		Ok((user_session.user_id, auth))
	}
}
