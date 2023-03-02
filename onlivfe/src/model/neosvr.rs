//! `NeosVR` related onlivfe models

impl From<neos::id::User> for super::PlatformAccountId {
	fn from(id: neos::id::User) -> Self { Self::NeosVR(id) }
}
impl From<&neos::id::User> for super::PlatformType {
	fn from(_: &neos::id::User) -> Self { Self::NeosVR }
}
impl From<neos::id::User> for super::PlatformType {
	fn from(id: neos::id::User) -> Self { Self::from(&id) }
}
