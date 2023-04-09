crate::platform_id!(
	/// The platform specific username/id/account.
	PlatformAccountId {
		vrc::id::User,
		chilloutvr::id::User,
		neos::id::User
	}
);

crate::platform_enum!(
	/// Details of a platform account
	PlatformAccount {
		Box<vrc::model::AnyUser>,
		Box<chilloutvr::model::UserDetails>,
		Box<neos::model::User>
	}
);
crate::platform_enum_id!(PlatformAccountId, PlatformAccount {
	v.data.as_user().base.id.clone(),
	v.data.base.id.clone(),
	v.data.id.clone()
} v);

crate::platform_enum!(
	/// Details of a platform account friend
	PlatformFriend {
		Box<vrc::model::Friend>,
		Box<chilloutvr::model::Friend>,
		Box<neos::model::Friend>
	}
);
crate::platform_enum_id!(PlatformAccountId, PlatformFriend {
	v.data.base.id.clone(),
	v.data.base.id.clone(),
	v.data.id.clone()
} v);
