//! The storage interface that core will use

/// A value to be stored
pub enum StoreOperation {
	/// Store platform account details
	Account(crate::model::PlatformAccount),
	/// Store profile details
	Profile(crate::model::Profile),
}

/// Storage query operation, with input and output
pub enum QueryOperation<'a> {
	/// Query for an account
	Account(
		(
			crate::model::PlatformAccountId,
			&'a mut Option<crate::model::PlatformAccount>,
		),
	),
}

#[async_trait::async_trait]
/// Storage backend for onlivfe
pub trait OnlivfeStore {
	/// The error type for operations using this storage backend
	type Err: std::error::Error;

	/// Stores a value into the storage backend
	async fn store(value: StoreOperation) -> Result<(), Self::Err>;
	/// Retrieves a value from the storage backend
	async fn query(query: QueryOperation) -> Result<(), Self::Err>;
}
