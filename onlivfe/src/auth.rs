use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::PlatformAccountId;

/// An error that occurred with the login
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "error", content = "data")]
pub enum LoginError {
	/// An error occurred
	Error(String),
	/// The authentication was partially successful, but requires additional
	/// verification
	RequiresAdditionalFactor(PlatformAccountId),
}

impl Display for LoginError {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match self {
					Self::Error(v) => write!(f, "{v}"),
					Self::RequiresAdditionalFactor(id) => write!(f, "2FA is required for account with ID '{}'", id.id_as_string())
				}
		}
}

crate::platform_enum!(
	/// Credentials for a platform
	#[derive(Eq)]
	Authentication {
		Box<vrc::query::Authentication>,
		Box<chilloutvr::query::SavedLoginCredentials>,
		Box<resonite::query::Authentication>
	}
);
crate::platform_enum_id!(PlatformAccountId, Authentication {
	v.metadata.updated_by.clone(),
	v.metadata.updated_by.clone(),
	v.data.user_id.clone()
} v);

// Can't use platform enum due to not knowing user IDs before auth has completed
#[derive(
	Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "platform", content = "data")]
/// Required for trying to create a platform authentication
pub enum LoginCredentials {
	/// VRC variant
	VRChat(Box<crate::vrchat::LoginRequestPart>),
	/// CVR variant
	ChilloutVR(Box<chilloutvr::query::LoginCredentials>),
	/// Resonite variant
	Resonite(Box<resonite::query::UserSessionQueryWithHeaders>),
}
crate::platform_specific!(LoginCredentials);

impl LoginCredentials {
	/// Sets the inner username/email/userid as a string
	///
	/// # Errors
	/// If parsing the value into a valid identifier failed for the current login
	/// credentials
	pub fn set_identifier(
		&mut self, value: String,
	) -> Result<(), std::io::Error> {
		match self {
			Self::VRChat(login_request_part) => match &mut **login_request_part {
				crate::vrchat::LoginRequestPart::LoginRequest(authenticating) => {
					authenticating.username = value;

					Ok(())
				}
				crate::vrchat::LoginRequestPart::SecondFactor((user_id, _)) => {
					match value.parse::<crate::vrchat::id::User>() {
						Ok(id) => {
							*user_id = id;

							Ok(())
						}
						Err(e) => {
							Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_owned()))
						}
					}
				}
			},
			Self::ChilloutVR(login_credentials) => {
				login_credentials.email = value;

				Ok(())
			}
			Self::Resonite(user_session_query_with_headers) => {
				match &mut user_session_query_with_headers.body.identifier {
					resonite::query::LoginCredentialsIdentifier::Username(v)
					| resonite::query::LoginCredentialsIdentifier::OwnerID(v)
					| resonite::query::LoginCredentialsIdentifier::Email(v) => {
						*v = value;

						Ok(())
					}
				}
			}
		}
	}

	#[must_use]
	/// Gets the inner username/email/userid as a string
	pub fn identifier(&self) -> &str {
		match self {
			Self::VRChat(login_request_part) => match &**login_request_part {
				crate::vrchat::LoginRequestPart::LoginRequest(authenticating) => {
					&authenticating.username
				}
				crate::vrchat::LoginRequestPart::SecondFactor((user_id, _)) => {
					user_id.as_ref()
				}
			},
			Self::ChilloutVR(login_credentials) => &login_credentials.email,
			Self::Resonite(user_session_query_with_headers) => {
				match &user_session_query_with_headers.body.identifier {
					resonite::query::LoginCredentialsIdentifier::Username(v)
					| resonite::query::LoginCredentialsIdentifier::OwnerID(v)
					| resonite::query::LoginCredentialsIdentifier::Email(v) => v,
				}
			}
		}
	}

	/// Sets the inner password/session token string
	///
	/// # Errors
	/// If not applicable to the current stage of authentication.
	pub fn set_primary_secret(
		&mut self, value: String,
	) -> Result<(), std::io::Error> {
		match self {
			Self::VRChat(login_request_part) => match &mut **login_request_part {
				crate::vrchat::LoginRequestPart::LoginRequest(authenticating) => {
					authenticating.password = value;

					Ok(())
				}
				crate::vrchat::LoginRequestPart::SecondFactor((_user_id, _)) => {
					Err(std::io::Error::new(
						std::io::ErrorKind::Other,
						"VRChat auth second factor stage doesn't have a primary secret"
							.to_owned(),
					))
				}
			},
			Self::ChilloutVR(login_credentials) => {
				login_credentials.password = value;

				Ok(())
			}
			Self::Resonite(user_session_query_with_headers) => {
				match &mut user_session_query_with_headers.body.authentication {
					resonite::query::UserSessionAuthentication::Password(v) => {
						v.password = value;

						Ok(())
					}
					resonite::query::UserSessionAuthentication::SessionToken(v) => {
						v.session_token = value;

						Ok(())
					}
				}
			}
		}
	}

	#[must_use]
	/// Gets the inner password/token as a string
	pub fn primary_secret(&self) -> Option<&str> {
		match self {
			Self::VRChat(login_request_part) => match &**login_request_part {
				crate::vrchat::LoginRequestPart::LoginRequest(authenticating) => {
					Some(&authenticating.password)
				}
				crate::vrchat::LoginRequestPart::SecondFactor((_user_id, _2fa)) => None,
			},
			Self::ChilloutVR(login_credentials) => Some(&login_credentials.password),
			Self::Resonite(user_session_query_with_headers) => {
				Some(match &user_session_query_with_headers.body.authentication {
					resonite::query::UserSessionAuthentication::Password(v) => {
						v.password.as_str()
					}
					resonite::query::UserSessionAuthentication::SessionToken(v) => {
						v.session_token.as_str()
					}
				})
			}
		}
	}

	/// Sets the inner 2FA/etc token string
	///
	/// # Errors
	/// If not applicable to the current stage of authentication.
	pub fn set_secondary_secret(
		&mut self, value: Option<String>,
	) -> Result<(), std::io::Error> {
		match self {
			Self::VRChat(login_request_part) => match &mut **login_request_part {
				crate::vrchat::LoginRequestPart::LoginRequest(authenticating) => {
					Err(std::io::Error::new(
						std::io::ErrorKind::Other,
						"VRChat auth first stage doesn't have a secondary secret"
							.to_owned(),
					))
				}
				crate::vrchat::LoginRequestPart::SecondFactor((
					_user_id,
					second_factor,
				)) => value.map_or_else(|| Err(std::io::Error::new(
						std::io::ErrorKind::Other,
						"VRChat second factor stage requires a secondary secret".to_owned(),
					)), |value| match second_factor {
							vrc::query::VerifySecondFactor::Email(code)
							| vrc::query::VerifySecondFactor::Code(code)
							| vrc::query::VerifySecondFactor::Recovery(code) => {
								*code = value;

								Ok(())
						}
					}),
			},
			Self::ChilloutVR(login_credentials) => Err(std::io::Error::new(
				std::io::ErrorKind::Other,
				"ChilloutVR doesn't support a secondary secret".to_owned(),
			)),
			Self::Resonite(user_session_query_with_headers) => {
				user_session_query_with_headers.data.second_factor = value;

				Ok(())
			}
		}
	}

	#[must_use]
	/// Gets the inner 2FA/etc token as a string
	pub fn secondary_secret(&self) -> Option<&str> {
		match self {
			Self::VRChat(login_request_part) => match &**login_request_part {
				crate::vrchat::LoginRequestPart::LoginRequest(_auth) => None,
				crate::vrchat::LoginRequestPart::SecondFactor((_, second_factor)) => {
					Some(match &second_factor {
						vrc::query::VerifySecondFactor::Email(code)
						| vrc::query::VerifySecondFactor::Code(code)
						| vrc::query::VerifySecondFactor::Recovery(code) => code,
					})
				}
			},
			Self::ChilloutVR(_creds) => None,
			Self::Resonite(user_session_query_with_headers) => {
				user_session_query_with_headers.data.second_factor.as_deref()
			}
		}
	}
}
