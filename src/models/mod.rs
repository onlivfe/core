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
	VRChat(String),
	ChilloutVR(String),
	NeosVR(neos::id::User),
}

pub enum PlatformAccount {
	VRChat(vrchatapi::models::User),
	ChilloutVR(chilloutvr::UserDetails),
	NeosVR(neos::User),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericAccount {
	/// The sharing ID of the account
	id: uuid::Uuid,
	display_name: String,
	ico_url: String,
	account_type: PlatformAccountIdDiscriminants,
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
	pub nick: Option<String>,
	pub notes: Option<String>,
	pub pfp_href: Option<String>,
}
