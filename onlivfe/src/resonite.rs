//! `Resonite` related onlivfe models

pub use resonite::*;

impl From<resonite::id::User> for super::PlatformAccountId {
	fn from(id: resonite::id::User) -> Self { Self::Resonite(id) }
}
impl From<&resonite::id::User> for super::PlatformType {
	fn from(_: &resonite::id::User) -> Self { Self::Resonite }
}
impl From<resonite::id::User> for super::PlatformType {
	fn from(id: resonite::id::User) -> Self { Self::from(&id) }
}

impl From<super::PlatformDataMetadata<resonite::id::User>>
	for super::PlatformDataMetadata<super::PlatformAccountId>
{
	fn from(value: super::PlatformDataMetadata<resonite::id::User>) -> Self {
		Self { updated_at: value.updated_at, updated_by: value.updated_by.into() }
	}
}
