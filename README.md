# ⚠️  This project is still work in progress and not in a useable state, yet. Please don't report missing or broken features. ⚠️

# Tusker

PostgreSQL specific migration tool

## Elevator pitch

Do you want to write your database schema directly as SQL which is understood by PostgreSQL?

Do you want to be able to make changes to this schema and generate the SQL which is required to migrate between the old and new schema version?

Tusker does exactly this.

## Progress

This project aims to replace the [Python version of Tusker][pypi-tusker] with
a complete rewrite in Rust. This project is in a very early stage and not ready
to be used.

- [ ] Diffing
  - [ ] collations
  - [ ] constraints
  - [ ] deps
  - [ ] domains
  - [ ] enums
  - [ ] extensions
  - [ ] functions
  - [ ] indexes
  - [ ] privileges
  - [ ] relations
  - [ ] rlspolicies
  - [ ] schemas
  - [ ] sequences
  - [ ] triggers
  - [ ] types

## Installation of the command line tool

```shell
cargo install tusker
```

Now you should be able to run tusker. Give it a try:

```
tusker --help
```

## Getting started

> ⚠️ **WARNING**  
> Diffing is not working, yet. This documentation is merely a placeholder for how things are supposed to work in the future.

Once tusker is installed create a new file called `schema.sql`:

```sql
CREATE TABLE fruit (
    id BIGINT GENERATED BY DEFAULT AS IDENTITY,
    name TEXT NOT NULL UNIQUE
);
```

You probably want to create an empty `migrations` directory, too:

```shell
mkdir migrations
```

Now you should be able to create your first migration:

```
tusker diff
```

The migration is printed to the console and all you need to do is
copy and paste the output into a new file in the migrations directory.
Alternatively you can also pipe the output of `tusker diff` into the
target file:

```
tusker diff > migrations/0001_initial.sql
```

After that check that your `schema.sql` and your `migrations` are in sync:

```
tusker diff
```

This should give you an empty output. This means that there is no difference
between applying the migrations in order and the target schema.

Alternatively you can run the check command:

```
tusker check
```

If you want to change the schema in the future simply change the `schema.sql`
and run `tusker diff` to create the migration for you.

Give it a try and change the `schema.sql`:

```sql
CREATE TABLE fruit (
    id BIGINT GENERATED BY DEFAULT AS IDENTITY,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT ''
);
```

Create a new migration:

```
tusker diff > migrations/0002_fruit_color.sql
```

**Congratulations! You are now using SQL to write your migrations. You are no longer limited by a 3rd party data definition language or an object relational wrapper.**

## Configuration

In order to run tusker you do not need a configuration file. The following
defaults are assumed:

- The file containing your database schema is called `schema.sql`
- The directory containing the migrations is called `migrations`
- Your current user can connect to the database using a unix
  domain socket without a password.

You can also create a configuration file called `tusker.toml`. The default
configuration looks like that:

```toml
[database]
#host = ""
#port = 5432
#user = ""
#password = ""
dbname = "tusker"

[schema]
filename = "schema.sql"

[migrations]
filename = "migrations/*.sql"

[diff]
safe = false
privileges = false
```

Instead of the exploded form of `host`, `port`, etc. it
is also possible to pass a connection URL:

```toml
[database]
url = "postgresql:///my_awesome_db"
```

You can also override the configuration using environment variables:

```toml
export TUSKER_DATABASE_URL=postgresql:///some-db-host/some-db
```

## How can I use the generated SQL files?

The resulting SQL files can either be applied to the database by hand
or by using the built-in migration manager of tusker.

In order to apply all the migrations, that haven't been run, in order
just execute the following command:

```shell
tusker migrate
```

## How does it work?

Upon startup `tusker` reads all files from the `migrations` directory
and runs them on an empty database. Another empty database is created
and the target schema is created. Then those two schemas are
diffed using the excellent [migra](https://pypi.org/project/migra/)
tool and the output printed to the console.

## Tusker is `unsafe` by default

Unlike `migra` the `tusker` command by default does not throw an
exception when a `drop`-statement is generated. Always check your
generated migrations prior to running them. If you want the same
behavior as migra you can either use the `--safe` argument or set
the `migra.safe` configuration option to `True` in your `tusker.toml`
file.

## FAQ

### Is it possible to split the schema into multiple files?

Yes. This feature has been added in 0.3. You can now use `glob` patterns as
part of the `schema.filename` setting. e.g.:

```toml
[schema]
filename = "schema/*.sql"
```

As of 0.4.5 recursive glob patterns are supported as well:

```toml
[schema]
filename = "schema/**/*.sql"
```

### Is it possible to diff the schema and/or migrations against an existing database?

Yes. This feature has been added in 0.2. You can pass a `from` and `to`
argument to the `tusker diff` command. Check the output of `tusker diff --help` for
more details.

### Tusker printed an error and left the temporary databases behind. How can I remove them?

Run `tusker clean`. This will remove all databases which were created
by previous runs of tusker. Tusker only removes databases which are
marked with a `CREATED BY TUSKER` comment.

### What does the `dbname` setting in `tusker.toml` mean?

When diffing against a ready migrated database this database name is used. This
command will print out the difference between the current database schema and
the target schema:

```shell
tusker diff database
```

Tusker also needs to create temporary databases when diffing against the `schema`
and/or `migrations`. The two databases are called `{dbname}_{timestamp}_schema`
and `{dbname}_{timestamp}_migrations`.

## FAQ

### How does it differ from the Tusker at PyPI?

[Tusker was originally written in Python][pypi-tusker] with the only feature
being schema diffing. It relied solely on [migra] and [schemainspect] to
perform the actual diffing. This version of Tusker implements the diffing
from scratch and also provides a type safe query system.

[pypi-tusker]: https://github.com/olivierlacan/keep-a-changelog/releases/tag/v0.0.1
[migra]: https://pypi.org/project/migra/
[schemainspect]: https://pypi.org/project/schemainspect/

### How can I upgrade from the Python version of Tusker?

```
$ pipx uninstall tusker
$ cargo install tusker
```

The `migra` configuration directive was renamed to `diff`. Just rename `[migra]`
in your config file to `[diff]`, if you got one, and you should be all set.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.
