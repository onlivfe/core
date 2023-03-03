//! Onlivfe's data structures

use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use time::OffsetDateTime;

pub mod cvr;
pub mod neosvr;
pub mod vrchat;

/// The platform specific username/id/account.
#[derive(
	Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, EnumDiscriminants,
)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(derive(Serialize, Deserialize))]
#[strum_discriminants(name(PlatformType))]
#[serde(tag = "platform", content = "id")]
pub enum PlatformAccountId {
	/// The platform is VRChat
	VRChat(vrc::id::User),
	/// The platform is ChilloutVR
	ChilloutVR(chilloutvr::id::User),
	/// The platform is NeosVR
	NeosVR(neos::id::User),
}

/// Details of a platform accoun
#[derive(Debug, Clone)]
pub enum PlatformAccount {
	/// Details about a VRChat account
	VRChat(Box<vrc::model::User>),
	/// Details about a ChilloutVR account
	ChilloutVR(Box<chilloutvr::model::UserDetails>),
	/// Details about a NeosVR account
	NeosVR(Box<neos::model::User>),
}

impl PlatformAccount {
	/// Gets the ID of the account
	#[must_use]
	pub fn id(&self) -> PlatformAccountId {
		match &self {
			Self::VRChat(acc) => PlatformAccountId::VRChat(acc.id.clone()),
			Self::ChilloutVR(acc) => {
				PlatformAccountId::ChilloutVR(acc.base.id.clone())
			}
			Self::NeosVR(acc) => PlatformAccountId::NeosVR(acc.id.clone()),
		}
	}

	/// Gets the platform of the account
	#[must_use]
	pub const fn platform(&self) -> PlatformType {
		match &self {
			Self::VRChat(_) => PlatformType::VRChat,
			Self::ChilloutVR(_) => PlatformType::ChilloutVR,
			Self::NeosVR(_) => PlatformType::NeosVR,
		}
	}
}

/// Credentials for a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformAuthentication {
	/// Authentication for a VRChat account
	VRChat(Box<(vrc::id::User, vrc::query::Authentication)>),
	/// Authentication for a ChilloutVR account
	ChilloutVR(
		Box<(chilloutvr::id::User, chilloutvr::query::SavedLoginCredentials)>,
	),
	/// Authentication for a NeosVR account
	NeosVR(Box<neos::query::Authentication>),
}

impl PlatformAuthentication {
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
#[derive(Debug, Clone)]
pub enum PlatformLogin {
	/// Authentication request for a VRChat account
	VRChat(Box<(String, String)>),
	/// Authentication request for a ChilloutVR account
	ChilloutVR(Box<chilloutvr::query::LoginCredentials>),
	/// Authentication request for a NeosVR account
	NeosVR(Box<neos::query::LoginCredentials>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A general abstraction over different types of accounts
pub struct GenericAccount {
	#[serde(flatten)]
	/// The ID of the account
	pub id: PlatformAccountId,
	/// Display name
	pub display_name: String,
	/// Icon URL
	pub ico_url: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
/// An ID of a profile
pub struct ProfileId(uuid::Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A profile of "this is someone".
pub struct Profile {
	#[serde(skip)]
	/// Only used in internal DB joins of data
	pub primary_key: u32,
	/// Used for mapping/importing/exporting profiles
	pub sharing_id: ProfileId,
	/// Nickname of the peep
	pub nick: Option<String>,
	/// Notes about the peep
	pub notes: Option<String>,
	/// A custom profile picture about the peep
	pub pfp_url: Option<String>,
}

/// Metadata about the data from a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformDataMetadata {
	/// When the data was fetched
	pub updated_at: OffsetDateTime,
	/// Which account was used to fetch the data
	pub updated_by: PlatformAccountId,
}

/// Metadata about the data from a platform with the data
#[derive(Debug, Clone)]
pub struct PlatformDataAndMetadata<T> {
	/// The actual data itself
	pub data: T,
	/// The metadata about the data
	pub metadata: PlatformDataMetadata,
}
