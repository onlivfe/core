crate::platform_id!(
	/// The platform specific username/id/account.
	PlatformAccountId {
		vrc::id::User,
		chilloutvr::id::User,
		resonite::id::User
	}
);

crate::platform_enum!(
	/// Details of a platform account
	PlatformAccount {
		Box<vrc::model::AnyUser>,
		Box<chilloutvr::model::UserDetails>,
		Box<resonite::model::User>
	}
);
crate::platform_enum_id!(PlatformAccountId, PlatformAccount {
	v.data.as_user().base.id.clone(),
	v.data.base.id.clone(),
	v.data.id.clone()
} v);

crate::platform_enum!(
	/// Details of a platform account friend
	#[derive(Eq)]
	PlatformFriend {
		Box<vrc::model::Friend>,
		Box<chilloutvr::model::Friend>,
		Box<resonite::model::Contact>
	}
);
crate::platform_enum_id!(PlatformAccountId, PlatformFriend {
	v.data.base.id.clone(),
	v.data.base.id.clone(),
	v.data.id.clone()
} v);
