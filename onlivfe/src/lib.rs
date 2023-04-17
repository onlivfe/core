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
use strum::{AsRefStr, EnumCount, EnumIter, IntoEnumIterator};
use time::OffsetDateTime;

pub mod cvr;
pub mod neosvr;
pub mod storage;
pub mod vrchat;

mod accounts;
pub use accounts::*;
mod auth;
pub use auth::*;
mod instances;
pub use instances::*;
mod assets;
pub use assets::*;

/// The type of the platform/service/game/etc
#[derive(
	Debug,
	Clone,
	Copy,
	AsRefStr,
	EnumIter,
	EnumCount,
	Serialize,
	Deserialize,
	PartialEq,
	Eq,
	Hash,
)]
pub enum PlatformType {
	/// It's VRC
	VRChat,
	/// It's CVR
	ChilloutVR,
	/// It's Neos
	NeosVR,
}

/// Gets all the platforms
#[must_use]
pub fn platforms() -> Vec<PlatformType> { PlatformType::iter().collect() }

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

macro_rules! platform_id {
	($(#[$meta:meta])*
	$name:ident { $vrc:ty, $cvr:ty, $neos:ty }) => {
		$(#[$meta])*
		#[derive(Clone, Hash, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
		#[serde(tag = "platform", content = "id")]
		pub enum $name {
			/// VRChat variant of the ID
			VRChat($vrc),
			/// ChilloutVR variant of the ID
			ChilloutVR($cvr),
			/// NeosVR variant of the ID
			NeosVR($neos),
		}
		crate::platform_specific!($name);

		impl $name {
			/// Gets the string representation of the internal platform specific ID
			#[must_use]
			pub fn id_as_string(&self) -> String {
				match self {
					Self::VRChat(v) => v.to_string(),
					Self::ChilloutVR(v) => v.to_string(),
					Self::NeosVR(v) => v.as_ref().to_string(),
				}
			}
		}
	};
}
pub(crate) use platform_id;

macro_rules! platform_enum {
	($(#[$meta:meta])*
	$name:ident { $vrc:ty, $cvr:ty, $neos:ty }) => {
		$(#[$meta])*
		#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
		#[serde(tag = "platform", content = "data")]
		pub enum $name {
			/// VRChat variant
			VRChat(crate::PlatformDataAndMetadata<$vrc, vrc::id::User>),
			/// ChilloutVR variant
			ChilloutVR(crate::PlatformDataAndMetadata<$cvr, chilloutvr::id::User>),
			/// NeosVR variant
			NeosVR(crate::PlatformDataAndMetadata<$neos, neos::id::User>),
		}
		crate::platform_specific!($name);

		impl $name {
			/// Copies the platform metadata into a more generic format
			#[must_use]
			pub fn metadata(&self) -> crate::PlatformDataMetadata<crate::PlatformAccountId> {
				match self {
					Self::VRChat(v) => v.metadata.clone().into(),
					Self::ChilloutVR(v)  => v.metadata.clone().into(),
					Self::NeosVR(v)  => v.metadata.clone().into(),
				}
			}
		}
	};
}
pub(crate) use platform_enum;

macro_rules! platform_enum_id {
	($id:ty, $name:ty { $vrc:expr, $cvr:expr, $neos:expr } $local_var:ident) => {
		impl $name {
			/// Gets the ID
			#[must_use]
			pub fn id(&self) -> $id {
				match self {
					Self::VRChat($local_var) => <$id>::VRChat($vrc),
					Self::ChilloutVR($local_var) => <$id>::ChilloutVR($cvr),
					Self::NeosVR($local_var) => <$id>::NeosVR($neos),
				}
			}
		}
	};
}
pub(crate) use platform_enum_id;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
/// An ID of a profile
pub struct ProfileId(uuid::Uuid);

impl ProfileId {
	/// Creates a new profile ID
	#[must_use]
	// New and default have slightly different semantics, as the new will be
	// different each time unlike what would assume for default.
	#[allow(clippy::new_without_default)]
	#[cfg(feature = "rand_util")]
	pub fn new() -> Self { Self(uuid::Uuid::new_v4()) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A profile of "this is someone".
pub struct Profile {
	/// Used for mapping/importing/exporting profiles
	pub sharing_id: ProfileId,
	/// Nickname of the peep
	pub nick: Option<String>,
	/// Notes about the peep
	pub notes: Option<String>,
	/// A custom profile picture about the peep
	pub pfp_url: Option<String>,
}

impl Profile {
	/// Creates a new profile ID
	#[must_use]
	// New and default have slightly different semantics, as the new will be
	// different each time unlike what would assume for default.
	#[allow(clippy::new_without_default)]
	#[cfg(feature = "rand_util")]
	pub fn new() -> Self {
		Self {
			sharing_id: ProfileId::new(),
			nick: None,
			notes: None,
			pfp_url: None,
		}
	}
}

/// Metadata about the data from a platform
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformDataMetadata<Id> {
	/// When the data was fetched
	pub updated_at: OffsetDateTime,
	/// Which account was used to fetch the data
	pub updated_by: Id,
}

/// Metadata about the data from a platform with the data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformDataAndMetadata<T, Id> {
	/// The actual data itself
	pub data: T,
	/// The metadata about the data
	pub metadata: PlatformDataMetadata<Id>,
}

impl<T, Id> PlatformDataAndMetadata<T, Id> {
	/// Creates a new instance of this with the current timestamp
	pub fn new_now(data: T, updated_by: Id) -> Self {
		Self {
			data,
			metadata: PlatformDataMetadata {
				updated_at: OffsetDateTime::now_utc(),
				updated_by,
			},
		}
	}

	/// Borrows the data and metadata as a tuple
	pub const fn as_tuple(&self) -> (&T, &PlatformDataMetadata<Id>) {
		(&self.data, &self.metadata)
	}
}

impl<T, Id> From<(T, PlatformDataMetadata<Id>)>
	for PlatformDataAndMetadata<T, Id>
{
	fn from(value: (T, PlatformDataMetadata<Id>)) -> Self {
		Self { data: value.0, metadata: value.1 }
	}
}

impl<T, Id> From<PlatformDataAndMetadata<T, Id>>
	for (T, PlatformDataMetadata<Id>)
{
	fn from(value: PlatformDataAndMetadata<T, Id>) -> Self {
		(value.data, value.metadata)
	}
}
