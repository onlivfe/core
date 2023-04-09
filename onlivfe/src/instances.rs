crate::platform_id!(
	/// The platform specific instance/session ID.
	InstanceId {
		vrc::id::WorldInstance,
		chilloutvr::id::Instance,
		neos::id::Session
	}
);

crate::platform_enum!(
	/// The platform specific instance/session.
	Instance {
		vrc::model::Instance,
		chilloutvr::model::ExtendedInstanceDetails,
		neos::model::SessionInfo
	}
);
crate::platform_enum_id!(InstanceId, Instance {
	v.data.id.clone(),
	v.data.base.id.clone(),
	v.data.id.clone()
} v);
