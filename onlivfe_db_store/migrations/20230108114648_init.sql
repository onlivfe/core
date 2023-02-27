CREATE TABLE profiles(
	profile_pk INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	sharing_id TEXT NOT NULL,
	nick TEXT,
	notes TEXT,
	pfp_href TEXT,
	created_at DATETIME NOT NULL DEFAULT DATETIME('now')
);

CREATE TABLE platform_accounts(
	profile_pk INTEGER NOT NULL,
	platform_type TEXT NOT NULL
	CHECK(platform_type IN ('vrchat', 'chilloutvr', 'chilloutvr')),
	platform_id TEXT NOT NULL,
	connection_created_at DATETIME NOT NULL DEFAULT DATETIME('now'),

	PRIMARY KEY(profile_pk, platform_type, platform_id),
	FOREIGN KEY(profile_pk) REFERENCES profiles(profile_pk)
);

CREATE TABLE vrchat_accounts(
	platform_account_type TEXT NOT NULL
	GENERATED ALWAYS AS ("vrchat") VIRTUAL,
	vrchat_user_id TEXT NOT NULL,
	cache_requester_vrchat_user_id TEXT,
	cache_stored_at DATETIME NOT NULL DEFAULT DATETIME('now'),

	FOREIGN KEY(platform_account_type, vrchat_user_id)
	REFERENCES platform_accounts(platform_type, platform_id)
);

CREATE TABLE chilloutvr_accounts(
	platform_account_type TEXT NOT NULL
	GENERATED ALWAYS AS "chilloutvr" VIRTUAL,
	chilloutvr_user_id TEXT NOT NULL,
	cache_requester_chilloutvr_user_id TEXT,
	cache_stored_at DATETIME NOT NULL DEFAULT DATETIME('now'),

	FOREIGN KEY(platform_account_type, chilloutvr_user_id)
	REFERENCES platform_accounts(platform_type, platform_id)
);

CREATE TABLE neosvr_accounts(
	platform_account_type TEXT NOT NULL
	GENERATED ALWAYS AS ("neosvr") VIRTUAL,
	neosvr_user_id TEXT NOT NULL,
	cache_requester_neosvr_user_id TEXT,
	cache_stored_at DATETIME NOT NULL DEFAULT DATETIME('now'),

	name TEXT,
	registered_at DATETIME,
	is_verified BOOLEAN,

	FOREIGN KEY(platform_account_type, neosvr_user_id)
	REFERENCES platform_accounts(platform_type, platform_id)
);
