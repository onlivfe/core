use serde::{Deserialize, Serialize};

/// The platform specific instance/session ID.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id")]
pub enum WorldId {
	/// The platform is VRChat
	VRChat(vrc::id::World),
	/// The platform is ChilloutVR
	ChilloutVR(chilloutvr::id::Asset),
	/// The platform is NeosVR
	NeosVR(neos::id::Record),
}
crate::platform_specific!(WorldId);

/// The platform specific instance/session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id")]
pub enum World {
	/// The platform is VRChat
	VRChat(Box<vrc::model::World>),
	/// The platform is ChilloutVR
	ChilloutVR(Box<chilloutvr::model::WorldDetails>),
	/// The platform is NeosVR
	NeosVR(Box<neos::model::Record>),
}
crate::platform_specific!(World);

impl World {
	/// Gets the ID of the account
	#[must_use]
	pub fn id(&self) -> WorldId {
		match &self {
			Self::VRChat(world) => WorldId::VRChat(world.base.id.clone()),
			Self::ChilloutVR(world) => {
				WorldId::ChilloutVR(world.base.base.id.clone())
			}
			Self::NeosVR(record) => WorldId::NeosVR(record.id.clone()),
		}
	}
}

/// The platform specific instance/session ID.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id")]
pub enum AvatarId {
	/// The platform is VRChat
	VRChat(vrc::id::Avatar),
	/// The platform is ChilloutVR
	ChilloutVR(chilloutvr::id::Asset),
	/// The platform is NeosVR
	NeosVR(neos::id::Record),
}
crate::platform_specific!(AvatarId);

/// The platform specific instance/session.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id")]
pub enum Avatar {
	/// The platform is VRChat
	VRChat(vrc::model::Avatar),
	/// The platform is ChilloutVR
	ChilloutVR(Box<chilloutvr::model::AvatarDetails>),
	/// The platform is NeosVR
	NeosVR(Box<neos::model::Record>),
}
crate::platform_specific!(Avatar);

impl Avatar {
	/// Gets the ID of the account
	#[must_use]
	pub fn id(&self) -> AvatarId {
		match &self {
			Self::VRChat(avatar) => AvatarId::VRChat(avatar.id.clone()),
			Self::ChilloutVR(avatar) => {
				AvatarId::ChilloutVR(avatar.base.base.id.clone())
			}
			Self::NeosVR(record) => AvatarId::NeosVR(record.id.clone()),
		}
	}
}
