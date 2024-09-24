use resonite::{
	api_client::{
		ApiClient,
		AuthenticatedResonite,
		AuthenticatingResonite,
		UnauthenticatedResonite,
		UserSessionQueryWithHeaders,
	},
	id,
	model::{Contact, SessionInfo, User},
	query::{self, Authentication, LoginCredentialsIdentifier},
};

use crate::OnlivfeApiClient;

impl OnlivfeApiClient {
	#[instrument]
	pub(crate) async fn logout_resonite(
		&self, id: &id::User,
	) -> Result<(), String> {
		if let Some(_client) = self.resonite.write().await.remove(id) {
			// TODO: Logout request
			//client.query(query::DestroyUserSession).await.map_err(|e| {
			//	error!("Logout as {:?} failed: {:?}", id, e);
			//	"Logout request failed, removing account anyway".to_string()
			//})?;
			return Ok(());
		}
		warn!("Tried to log out of non-existent account {:?}", id);
		Ok(())
	}

	// Auth contains user ID so not passing it in here unlike other clients
	#[instrument]
	pub(crate) async fn reauthenticate_resonite(
		&self, auth: Authentication,
	) -> Result<(), String> {
		trace!("Reauthentcating as {:?}", &auth.user_id);
		if let Some(api) = self.resonite.read().await.get(&auth.user_id) {
			warn!(
				"Already had authenticated client for reauthentication as {:?}",
				auth.user_id
			);
			api.query(query::ExtendUserSession).await.map_err(|e| {
				warn!(
					"Reauthentication via user session extension check as {:?} failed: {:?}",
					auth.user_id, e
				);
				"Reauthentication failed".to_owned()
			})?;
		} else {
			let id = auth.user_id.clone();
			let mut rw_lock_guard = self.resonite.write().await;
			let api = AuthenticatedResonite::new(self.user_agent.clone(), auth)
				.map_err(|e| {
					error!("Creating Resonite API client as {id:?} failed: {e:?}");
					"Internal error, Resonite API client creation failed".to_string()
				})?;

			api.query(query::ExtendUserSession).await.map_err(|e| {
				warn!(
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
	pub(crate) async fn extend_auth_resonite(
		&self, id: &id::User,
	) -> Result<(), String> {
		let rw_lock_guard = self.resonite.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "Resonite API not authenticated".to_owned())?;
		api.query(query::ExtendUserSession).await.map_err(|e| {
			warn!("User session extension as {:?} failed: {:?}", id, e);
			"User session extension failed".to_owned()
		})?;

		Ok(())
	}

	#[instrument]
	pub(crate) async fn instance_resonite(
		&self, id: &id::User, session_id: id::Session,
	) -> Result<SessionInfo, String> {
		trace!("Fetching instance {:?} as {:?}", session_id, id);
		let rw_lock_guard = self.resonite.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "Resonite API not authenticated".to_owned())?;
		let query = query::SessionInfo { session_id };
		let session = api.query(query).await.map_err(|e| {
			warn!("Instance query failed: {:?}", &e);
			"Resonite instance query failed".to_owned()
		})?;

		Ok(session)
	}

	#[instrument]
	pub(crate) async fn user_resonite(
		&self, get_as: &id::User, user_id: id::User,
	) -> Result<User, String> {
		trace!("Fetching user {:?} as {:?}", user_id, get_as);
		let rw_lock_guard = self.resonite.read().await;
		let api = rw_lock_guard
			.get(get_as)
			.ok_or_else(|| "Resonite API not authenticated".to_owned())?;
		let query = query::UserInfo::new(user_id);
		let user = api.query(query).await.map_err(|e| {
			warn!("User query failed: {:?}", &e);
			"Resonite user query failed".to_owned()
		})?;

		Ok(user)
	}

	#[instrument]
	pub(crate) async fn contacts_resonite(
		&self, id: &id::User,
	) -> Result<Vec<Contact>, String> {
		trace!("Fetching Contacts as {:?}", id);
		let rw_lock_guard = self.resonite.read().await;
		let api = rw_lock_guard
			.get(id)
			.ok_or_else(|| "Resonite API not authenticated".to_owned())?;
		let query = query::Contacts;
		let contacts = api.query(query).await.map_err(|e| {
			warn!("Contacts query failed: {:?}", &e);
			"Resonite Contacts query failed".to_owned()
		})?;

		Ok(contacts)
	}

	#[instrument]
	pub(crate) async fn login_resonite(
		&self, auth: UserSessionQueryWithHeaders,
	) -> Result<(id::User, query::Authentication), String> {
		trace!("Trying to login as {:?}", auth.body.identifier);
		let mut rw_lock_guard = self.resonite.write().await;
		let api = match &auth.body.identifier {
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
				|| UnauthenticatedResonite::new(self.user_agent.clone()),
				AuthenticatedResonite::downgrade,
			)
			.map_err(|_| {
				"Internal error, Resonite API client creation failed".to_string()
			})?;
		let api = AuthenticatingResonite::from((api, auth.data));

		let result = api.query(auth.body).await.map_err(|e| {
			warn!("Login query failed: {:?}", &e);
			"Resonite authentication failed".to_owned()
		})?;
		trace!(
			"Auth request for {:?} was successful",
			&result.user_session.user_id
		);

		let user_id = result.user_session.user_id.clone();
		let auth = query::Authentication::from(result.user_session);

		let api = UnauthenticatedResonite::from(api)
			.upgrade(auth.clone())
			.map_err(|_| {
				"Internal error, authenticated Resonite API client's creation failed"
					.to_owned()
			})?;
		rw_lock_guard.insert(user_id.clone(), api);
		Ok((user_id, auth))
	}
}
