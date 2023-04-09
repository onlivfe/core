crate::platform_id!(
	/// The platform specific instance/session ID.
	WorldId {
		vrc::id::World,
		chilloutvr::id::Asset,
		neos::id::Record
	}
);

crate::platform_enum!(
	/// The platform specific instance/session.
	World {
		Box<vrc::model::World>,
		Box<chilloutvr::model::WorldDetails>,
		Box<neos::model::Record>
	}
);
crate::platform_enum_id!(WorldId, World {
	v.data.base.id.clone(),
	v.data.base.base.id.clone(),
	v.data.id.clone()
} v);

crate::platform_id!(
	/// The platform specific avatar ID.
	AvatarId {
		vrc::id::Avatar,
		chilloutvr::id::Asset,
		neos::id::Record
	}
);

crate::platform_enum!(
	/// The platform specific avatar.
	Avatar {
		Box<vrc::model::Avatar>,
		Box<chilloutvr::model::AvatarDetails>,
		Box<neos::model::Record>
	}
);
crate::platform_enum_id!(AvatarId, Avatar {
	v.data.id.clone(),
	v.data.base.base.id.clone(),
	v.data.id.clone()
} v);
