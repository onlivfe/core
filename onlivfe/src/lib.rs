//! The core models that power [onlivfe](https://onlivfe.com).
//!
//! Very WIP.

#![cfg_attr(nightly, feature(doc_auto_cfg))]
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![deny(clippy::cargo)]
#![warn(missing_docs)]
#![deny(rustdoc::invalid_html_tags)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// My project my choice, tabs are literally made for indentation, spaces not.
#![allow(clippy::tabs_in_doc_comments)]
// Not much can be done about it :/
#![allow(clippy::multiple_crate_versions)]

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumCount, EnumIter};
use time::OffsetDateTime;

pub mod cvr;
pub mod neosvr;
pub mod storage;
pub mod vrchat;

mod accounts;
pub use accounts::*;
mod instances;
pub use instances::*;
mod assets;
pub use assets::*;

/// The type of the platform/service/game/etc
#[derive(
	Debug, Clone, Copy, AsRefStr, EnumIter, EnumCount, Serialize, Deserialize,
)]
pub enum PlatformType {
	/// It's VRC
	VRChat,
	/// It's CVR
	ChilloutVR,
	/// It's Neos
	NeosVR,
}

macro_rules! platform_specific {
	($name:ident) => {
		impl From<&$name> for crate::PlatformType {
			fn from(value: &$name) -> Self {
				match value {
					$name::VRChat(_) => Self::VRChat,
					$name::ChilloutVR(_) => Self::ChilloutVR,
					$name::NeosVR(_) => Self::NeosVR,
				}
			}
		}

		impl From<$name> for crate::PlatformType {
			fn from(value: $name) -> Self { Self::from(&value) }
		}

		impl $name {
			/// Gets the platform type
			#[must_use]
			pub const fn platform(&self) -> crate::PlatformType {
				match &self {
					Self::VRChat(_) => crate::PlatformType::VRChat,
					Self::ChilloutVR(_) => crate::PlatformType::ChilloutVR,
					Self::NeosVR(_) => crate::PlatformType::NeosVR,
				}
			}
		}
	};
}

pub(crate) use platform_specific;

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
