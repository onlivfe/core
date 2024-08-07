crate::platform_id!(
	/// The platform specific instance/session ID.
	InstanceId {
		vrc::id::WorldInstance,
		chilloutvr::id::Instance,
		resonite::id::Session
	}
);

crate::platform_enum!(
	/// The platform specific instance/session.
	#[derive(Eq)]
	Instance {
		vrc::model::Instance,
		chilloutvr::model::ExtendedInstanceDetails,
		resonite::model::SessionInfo
	}
);
crate::platform_enum_id!(InstanceId, Instance {
	v.data.id.clone(),
	v.data.base.id.clone(),
	v.data.id.clone()
} v);
