

## useful links


* <https://docs.rs/sqlx/latest/>
    * <https://docs.rs/sqlx/latest/sqlx/trait.Type.html>
    * <https://docs.rs/sqlx/latest/sqlx/trait.FromRow.html#rename_all>
* <https://docs.rs/sqlx-postgres/latest/sqlx_postgres/>

## DB stuff

* one table
    * id
    * channel_ID, 24 char (0-9, a-z, A-Z, - and _)
    * channel_title: string? limit?

postgresql 15
docker: (cf script/start_db.sh)
init db:

* `sqlx database create` to create a database, relies on the `DATABASE_URL` env
variable `postgres://${DB_USER}:${DB_PASSWORD}@${DB_PORT}/${DB_NAME}`
* to create a migration: `sqlx migrate add <us_id>_<name_of_migration>`
    * it creates a directory `migrations`
    * and adds a file `{timestamp}_<name_of_migration>.sql`
* to run a migration: `sqlx migrate run`
* sqlx creates a table `sqlx_migrations` to keep track of already run.