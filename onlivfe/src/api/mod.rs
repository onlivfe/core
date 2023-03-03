//! Core's API connection handling

use chilloutvr::api_client::{AuthenticatedCVR, UnauthenticatedCVR};
use neos::api_client::{AuthenticatedNeos, UnauthenticatedNeos};
use tokio::sync::RwLock;
use vrc::api_client::{AuthenticatedVRC, UnauthenticatedVRC};

use crate::model::PlatformType;

enum PossiblyAuthenticated<Auth, NoAuth> {
	Authenticated(Auth),
	Unauthenticated(NoAuth),
}

impl<A, N> PossiblyAuthenticated<A, N> {
	pub fn is_authenticated(&self) -> bool {
		match &self {
			PossiblyAuthenticated::Authenticated(_) => true,
			PossiblyAuthenticated::Unauthenticated(_) => false,
		}
	}
}

/// An unified API client interface for the different platforms
pub struct OnlivfeApiClient {
	/// The VRChat API client
	vrc:
		RwLock<Option<PossiblyAuthenticated<AuthenticatedVRC, UnauthenticatedVRC>>>,
	/// The ChilloutVR API client
	cvr: RwLock<PossiblyAuthenticated<AuthenticatedCVR, UnauthenticatedCVR>>,
	/// The NeosVR API client
	neos: RwLock<PossiblyAuthenticated<AuthenticatedNeos, UnauthenticatedNeos>>,
}

impl OnlivfeApiClient {
	pub fn new(user_agent: String) -> Result<Self, String> {
		Ok(Self {
			vrc: RwLock::new(None),
			cvr: RwLock::new(PossiblyAuthenticated::Unauthenticated(
				UnauthenticatedCVR::new(user_agent.clone())
					.map_err(|_| "Failed to create Neos API client")?,
			)),
			neos: RwLock::new(PossiblyAuthenticated::Unauthenticated(
				UnauthenticatedNeos::new(user_agent)
					.map_err(|_| "Failed to create Neos API client")?,
			)),
		})
	}

	pub async fn is_authenticated(&self, platform: PlatformType) -> bool {
		match platform {
			PlatformType::VRChat => {
				let api = self.vrc.read().await;
				api
					.as_ref()
					.map_or_else(|| false, PossiblyAuthenticated::is_authenticated)
			}
			PlatformType::ChilloutVR => {
				let api = self.cvr.read().await;
				api.is_authenticated()
			}
			PlatformType::NeosVR => {
				let api = self.neos.read().await;
				api.is_authenticated()
			}
		}
	}

	pub async fn login(
		&self, auth: crate::model::PlatformLogin,
	) -> Result<(), String> {
		todo!();
	}

	pub async fn reauthenticate(
		&self, auth: crate::model::PlatformAuthentication,
	) -> Result<(), String> {
		todo!();
	}
}
