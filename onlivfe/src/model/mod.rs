//! Onlivfe's data structures

use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

pub mod cvr;
pub mod neosvr;
pub mod vrchat;

/// The platform specific username/id/account.
#[derive(Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize))]
#[serde(tag = "platform", content = "id")]
pub enum PlatformAccountId {
	/// A VRChat user's ID
	VRChat(vrc::id::User),
	/// A ChilloutVR user's ID
	ChilloutVR(chilloutvr::id::User),
	/// A NeosVR user's ID
	NeosVR(neos::id::User),
}

/// Details of a platform account
pub enum PlatformAccount {
	/// Details about a VRChat account
	VRChat(Box<vrc::model::User>),
	/// Details about a ChilloutVR account
	ChilloutVR(Box<chilloutvr::model::UserDetails>),
	/// Details about a NeosVR account
	NeosVR(Box<neos::model::User>),
}

/// Credentials for a platform
#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize))]
pub enum PlatformAuthentication {
	/// Authentication for a VRChat account
	VRChat(Box<String>),
	/// Authentication for a ChilloutVR account
	ChilloutVR(Box<chilloutvr::query::SavedLoginCredentials>),
	/// Authentication for a NeosVR account
	NeosVR(Box<neos::query::Authentication>),
}

/// Struct required for trying to create a platform authentication
#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize))]
pub enum PlatformLogin {
	/// Authentication request for a VRChat account
	VRChat(Box<String>),
	/// Authentication request for a ChilloutVR account
	ChilloutVR(Box<chilloutvr::query::LoginCredentials>),
	/// Authentication request for a NeosVR account
	NeosVR(Box<neos::query::LoginCredentials>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A general abstraction over different types of accounts
pub struct GenericAccount {
	#[serde(flatten)]
	/// The ID of the account
	id: PlatformAccountId,
	/// Display name
	display_name: String,
	/// Icon URL
	ico_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A profile of "this is someone".
pub struct Profile {
	#[serde(skip)]
	/// Only used in internal DB joins of data
	pub primary_key: u32,
	/// Used for mapping/importing/exporting profiles
	pub sharing_id: uuid::Uuid,
	/// Nickname of the peep
	pub nick: Option<String>,
	/// Notes about the peep
	pub notes: Option<String>,
	/// A custom profile picture about the peep
	pub pfp_url: Option<String>,
}
