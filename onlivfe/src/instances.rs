use serde::{Deserialize, Serialize};

/// The platform specific instance/session ID.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id")]
pub enum InstanceId {
	/// The platform is VRChat
	VRChat(vrc::id::WorldInstance),
	/// The platform is ChilloutVR
	ChilloutVR(chilloutvr::id::Instance),
	/// The platform is NeosVR
	NeosVR(neos::id::Session),
}
crate::platform_specific!(InstanceId);

/// The platform specific instance/session.
#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
#[serde(tag = "platform", content = "id")]
pub enum Instance {
	/// The platform is VRChat
	VRChat(vrc::model::Instance),
	/// The platform is ChilloutVR
	ChilloutVR(chilloutvr::model::ExtendedInstanceDetails),
	/// The platform is NeosVR
	NeosVR(neos::model::SessionInfo),
}
crate::platform_specific!(Instance);

impl Instance {
	/// Gets the ID of the account
	#[must_use]
	pub fn id(&self) -> InstanceId {
		match &self {
			Self::VRChat(instance) => InstanceId::VRChat(instance.id.clone()),
			Self::ChilloutVR(instance) => {
				InstanceId::ChilloutVR(instance.base.id.clone())
			}
			Self::NeosVR(session) => InstanceId::NeosVR(session.id.clone()),
		}
	}
}
