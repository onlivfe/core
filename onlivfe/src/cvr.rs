//! `ChilloutVR` related onlivfe models

pub use chilloutvr::*;

impl From<chilloutvr::id::User> for super::PlatformAccountId {
	fn from(id: chilloutvr::id::User) -> Self { Self::ChilloutVR(id) }
}

impl From<&chilloutvr::id::User> for super::PlatformType {
	fn from(_: &chilloutvr::id::User) -> Self { Self::ChilloutVR }
}
impl From<chilloutvr::id::User> for super::PlatformType {
	fn from(id: chilloutvr::id::User) -> Self { Self::from(&id) }
}
