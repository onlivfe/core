//! `NeosVR` related onlivfe models

pub use neos::*;

impl From<neos::id::User> for super::PlatformAccountId {
	fn from(id: neos::id::User) -> Self { Self::NeosVR(id) }
}
impl From<&neos::id::User> for super::PlatformType {
	fn from(_: &neos::id::User) -> Self { Self::NeosVR }
}
impl From<neos::id::User> for super::PlatformType {
	fn from(id: neos::id::User) -> Self { Self::from(&id) }
}

impl From<super::PlatformDataMetadata<neos::id::User>>
	for super::PlatformDataMetadata<super::PlatformAccountId>
{
	fn from(value: super::PlatformDataMetadata<neos::id::User>) -> Self {
		Self { updated_at: value.updated_at, updated_by: value.updated_by.into() }
	}
}
