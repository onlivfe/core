//! `NeosVR` related onlivfe models

impl From<neos::id::User> for super::PlatformAccountId {
	fn from(id: neos::id::User) -> Self { Self::NeosVR(id) }
}
