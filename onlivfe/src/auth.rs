use crate::PlatformAccountId;

crate::platform_enum!(
	/// Credentials for a platform
	Authentication {
		Box<vrc::query::Authentication>,
		Box<chilloutvr::query::SavedLoginCredentials>,
		Box<neos::query::Authentication>
	}
);
crate::platform_enum_id!(PlatformAccountId, Authentication {
	v.metadata.updated_by.clone(),
	v.metadata.updated_by.clone(),
	v.data.user_id.clone()
} v);

// Can't use platform enum due to not knowing user IDs before auth has completed
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "platform", content = "data")]
/// Required for trying to create a platform authentication
pub enum LoginCredentials {
	/// VRChat variant
	VRChat(Box<crate::vrchat::LoginRequestPart>),
	/// ChilloutVR variant
	ChilloutVR(Box<chilloutvr::query::LoginCredentials>),
	/// NeosVR variant
	NeosVR(Box<neos::query::LoginCredentials>),
}
crate::platform_specific!(LoginCredentials);
