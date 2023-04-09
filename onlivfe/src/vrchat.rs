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

/// A VRC login request portion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginRequestPart {
	/// Login with credentials
	LoginRequest(vrc::query::Authenticating),
	/// Continuing authentication with second factor
	SecondFactor((vrc::id::User, vrc::query::VerifySecondFactor)),
}
