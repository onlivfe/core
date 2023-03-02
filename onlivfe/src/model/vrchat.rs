//! `VRChat` related onlivfe models

impl From<vrc::id::User> for super::PlatformAccountId {
	fn from(id: vrc::id::User) -> Self { Self::VRChat(id) }
}
impl From<&vrc::id::User> for super::PlatformType {
	fn from(_: &vrc::id::User) -> Self { Self::VRChat }
}
impl From<vrc::id::User> for super::PlatformType {
	fn from(id: vrc::id::User) -> Self { Self::from(&id) }
}
