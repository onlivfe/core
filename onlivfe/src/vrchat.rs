//! `VRChat` related onlivfe models

use serde::{Deserialize, Serialize};
pub use vrc::*;

impl From<vrc::id::User> for super::PlatformAccountId {
	fn from(id: vrc::id::User) -> Self { Self::VRChat(id) }
}
impl From<&vrc::id::User> for super::PlatformType {
	fn from(_: &vrc::id::User) -> Self { Self::VRChat }
}
impl From<vrc::id::User> for super::PlatformType {
	fn from(id: vrc::id::User) -> Self { Self::from(&id) }
}

impl From<super::PlatformDataMetadata<vrc::id::User>>
	for super::PlatformDataMetadata<super::PlatformAccountId>
{
	fn from(value: super::PlatformDataMetadata<vrc::id::User>) -> Self {
		Self { updated_at: value.updated_at, updated_by: value.updated_by.into() }
	}
}

/// A VRC login request portion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginRequestPart {
	/// Login with credentials
	LoginRequest(vrc::query::Authenticating),
	/// Continuing authentication with second factor
	SecondFactor((vrc::id::User, vrc::query::VerifySecondFactor)),
}
