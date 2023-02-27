# `onlivfe_db_store`

The database storage backend of onlivfe.
One ready made option for `core`'s storage backend, utilizing an SQLite database using `SQLx`.

[SQLx CLI](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) (`cargo install sqlx-cli --no-default-features --features sqlite`) can be very useful when working on this.

## Database changes

Generally speaking, you first generate a new migration:

```sh
sqlx migrate add NAME_OF_MIGRATION
```

Then write the SQL changes, and then re-generate the schema by running the migrations:

```sh
# Drop DB, recreate it and run all migrations
sqlx database drop && sqlx database create && sqlx migrate run
# Save metadata
cargo sqlx prepare
```
