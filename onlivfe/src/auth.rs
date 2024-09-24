use serde::{Deserialize, Serialize};

use crate::PlatformAccountId;

/// An error that occurred with the login
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "error", content = "data")]
pub enum LoginError {
	/// An error occurred
	Error(String),
	/// The authentication was partially successful, but requires additional
	/// verification
	RequiresAdditionalFactor(PlatformAccountId),
}

crate::platform_enum!(
	/// Credentials for a platform
	#[derive(Eq)]
	Authentication {
		Box<vrc::query::Authentication>,
		Box<chilloutvr::query::SavedLoginCredentials>,
		Box<resonite::query::Authentication>
	}
);
crate::platform_enum_id!(PlatformAccountId, Authentication {
	v.metadata.updated_by.clone(),
	v.metadata.updated_by.clone(),
	v.data.user_id.clone()
} v);

// Can't use platform enum due to not knowing user IDs before auth has completed
#[derive(
	Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "platform", content = "data")]
/// Required for trying to create a platform authentication
pub enum LoginCredentials {
	/// VRC variant
	VRChat(Box<crate::vrchat::LoginRequestPart>),
	/// CVR variant
	ChilloutVR(Box<chilloutvr::query::LoginCredentials>),
	/// Resonite variant
	Resonite(Box<resonite::api_client::UserSessionQueryWithHeaders>),
}
crate::platform_specific!(LoginCredentials);
