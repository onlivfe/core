use serde::{Deserialize, Serialize};

/// The platform specific username/id/account.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "platform", content = "user_id")]
pub enum PlatformAccountId {
	/// The platform is VRChat
	VRChat(vrc::id::User),
	/// The platform is ChilloutVR
	ChilloutVR(chilloutvr::id::User),
	/// The platform is NeosVR
	NeosVR(neos::id::User),
}
crate::platform_specific!(PlatformAccountId);

/// Details of a platform account
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", content = "user")]
pub enum PlatformAccount {
	/// Details about a VRChat account
	VRChat(vrc::model::AnyUser),
	/// Details about a ChilloutVR account
	ChilloutVR(Box<chilloutvr::model::UserDetails>),
	/// Details about a NeosVR account
	NeosVR(Box<neos::model::User>),
}
crate::platform_specific!(PlatformAccount);

impl PlatformAccount {
	/// Gets the ID of the account
	#[must_use]
	pub fn id(&self) -> PlatformAccountId {
		match &self {
			Self::VRChat(acc) => {
				PlatformAccountId::VRChat(acc.as_user().base.id.clone())
			}
			Self::ChilloutVR(acc) => {
				PlatformAccountId::ChilloutVR(acc.base.id.clone())
			}
			Self::NeosVR(acc) => PlatformAccountId::NeosVR(acc.id.clone()),
		}
	}
}

/// Details of a platform account friend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", content = "friend")]
pub enum PlatformFriend {
	/// Details about a VRChat account
	VRChat(vrc::model::Friend),
	/// Details about a ChilloutVR account
	ChilloutVR(chilloutvr::model::Friend),
	/// Details about a NeosVR account
	NeosVR(neos::model::Friend),
}
crate::platform_specific!(PlatformFriend);

impl PlatformFriend {
	/// Get the ID of the platform friend
	#[must_use]
	pub fn id(&self) -> PlatformAccountId {
		match self {
			Self::VRChat(t) => PlatformAccountId::from(t.base.id.clone()),
			Self::ChilloutVR(t) => PlatformAccountId::from(t.base.id.clone()),
			Self::NeosVR(v) => PlatformAccountId::from(v.id.clone()),
		}
	}
}

/// Credentials for a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", content = "auth")]
pub enum Authentication {
	/// Authentication for a VRChat account
	VRChat(Box<(vrc::id::User, vrc::query::Authentication)>),
	/// Authentication for a ChilloutVR account
	ChilloutVR(
		Box<(chilloutvr::id::User, chilloutvr::query::SavedLoginCredentials)>,
	),
	/// Authentication for a NeosVR account
	NeosVR(Box<neos::query::Authentication>),
}
crate::platform_specific!(Authentication);

impl Authentication {
	/// Get the ID of the platform account
	#[must_use]
	pub fn id(&self) -> PlatformAccountId {
		match self {
			Self::VRChat(t) => PlatformAccountId::from(t.0.clone()),
			Self::ChilloutVR(t) => PlatformAccountId::from(t.0.clone()),
			Self::NeosVR(v) => PlatformAccountId::from(v.user_id.clone()),
		}
	}
}

/// Struct required for trying to create a platform authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "platform", content = "login")]
pub enum LoginCredentials {
	/// Authentication request for a VRChat account, or a 2FA token
	VRChat(Box<crate::vrchat::LoginRequestPart>),
	/// Authentication request for a ChilloutVR account
	ChilloutVR(Box<chilloutvr::query::LoginCredentials>),
	/// Authentication request for a NeosVR account
	NeosVR(Box<neos::query::LoginCredentials>),
}
crate::platform_specific!(LoginCredentials);
