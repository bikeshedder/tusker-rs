CREATE EXTENSION IF NOT EXISTS btree_gist;

BEGIN;

CREATE TYPE migration_operation AS ENUM (
    'apply',
    'fake',
    'update',
    'delete'
);

CREATE TABLE IF NOT EXISTS "migration" (
    "number" INTEGER NOT NULL,
    "name" text NOT NULL DEFAULT '',
    "hash" bytea,
    "validity" tstzrange NOT NULL DEFAULT tstzrange(now(), NULL),
    "operation" migration_operation NOT NULL DEFAULT 'apply',
    "comment" text NOT NULL DEFAULT '',
    EXCLUDE USING GIST ("number" WITH =, "validity" WITH &&)
);

CREATE INDEX "validity_idx" ON "migration" USING GIST ("number", "validity");

CREATE VIEW migration_current AS
SELECT "number",
    "name",
    "hash",
    lower("validity") AS "applied"
FROM migration
WHERE now() <@ "validity"
    AND "operation" != 'delete'
ORDER BY "number";

CREATE VIEW migration_log AS
SELECT "number",
    "name",
    lower("validity") AS "timestamp",
    "operation"
FROM "migration"
ORDER BY "timestamp";

CREATE FUNCTION migration_insert(
    n INTEGER,
    NAME text,
    HASH bytea,
    operation migration_operation DEFAULT 'apply'
) RETURNS VOID AS '
    INSERT INTO "migration" ("number", name, hash, operation)
    VALUES (n, name, hash, operation);
' LANGUAGE SQL;

CREATE FUNCTION migration_fake(n INTEGER, NAME text, HASH bytea) RETURNS VOID AS '
    SELECT migration_insert(n, name, hash, ''fake'');
' LANGUAGE SQL;

CREATE FUNCTION _migration_delete(n INTEGER) RETURNS VOID AS '
    UPDATE "migration"
    SET "validity" = tstzrange(lower("validity"), now())
    WHERE now() <@ "validity" AND "migration"."number" = n
' LANGUAGE SQL;

CREATE FUNCTION migration_update(
    n INTEGER,
    NAME text,
    HASH bytea,
    operation migration_operation DEFAULT 'update'
) RETURNS VOID AS '
    SELECT _migration_delete(n);
    SELECT migration_insert(n, name, hash, operation);
' LANGUAGE SQL;

CREATE FUNCTION migration_delete(n INTEGER) RETURNS VOID AS '
    SELECT migration_update(n, (SELECT name FROM migration_current WHERE "number"=n), NULL, ''delete'');
' LANGUAGE SQL;

END;
