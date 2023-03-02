fn main() {
	println!("cargo:rerun-if-changed=migrations");

	if std::env::var_os("DOCS_RS").is_some() {
		println!("cargo:rustc-env=SQLX_OFFLINE=true");
	}
}
