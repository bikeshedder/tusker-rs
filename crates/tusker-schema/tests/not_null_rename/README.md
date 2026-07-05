# not_null_rename

Renaming a column keeps the name of its backing `NOT NULL` constraint derived
from the *old* column name (PostgreSQL 18+), so `tusker diff` must emit a
`RENAME CONSTRAINT` to reconcile the drift. Named `NOT NULL` constraints only
exist on PostgreSQL 18 and later, hence the `min_server_version` in `test.toml`.
