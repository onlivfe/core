//! The DB storage backend that can be used to power [onlivfe](https://onlivfe.com).
//!
//! Very WIP.

#![cfg_attr(nightly, feature(doc_cfg))]
#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![deny(clippy::cargo)]
#![warn(missing_docs)]
#![deny(rustdoc::invalid_html_tags)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// My project my choice, tabs are literally made for indentation, spaces not.
#![allow(clippy::tabs_in_doc_comments)]
// Not much can be done about it :/
#![allow(clippy::multiple_crate_versions)]

/// An in-memory only cache storage backend for onlivfe
#[derive(Clone, Default)]
pub struct OnlivfeCacheStorageBackend {}

impl OnlivfeCacheStorageBackend {
	#[must_use]
	/// Creates a new onlivfe cache storage backend
	pub fn new() -> Self { Self::default() }
}
