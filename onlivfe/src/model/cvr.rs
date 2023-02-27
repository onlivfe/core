//! `ChilloutVR` related onlivfe models

impl From<chilloutvr::id::User> for super::PlatformAccountId {
	fn from(id: chilloutvr::id::User) -> Self { Self::ChilloutVR(id) }
}
