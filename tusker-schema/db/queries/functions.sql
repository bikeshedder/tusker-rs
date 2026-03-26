SELECT
    ns.nspname AS schema,
    p.proname AS name,
    p.prokind AS kind,
    pg_get_function_identity_arguments(p.oid) AS identity_arguments,
    pg_get_functiondef(p.oid) AS definition
FROM pg_catalog.pg_proc AS p
JOIN pg_catalog.pg_namespace AS ns ON ns.oid = p.pronamespace
WHERE ns.nspname = $1
  AND p.prokind = 'f'
  -- Skip routines that belong to an installed extension. Those should be
  -- managed via CREATE EXTENSION / ALTER EXTENSION, not by schema diffs.
  AND NOT EXISTS (
      SELECT 1
      FROM pg_catalog.pg_depend AS dep
      WHERE dep.classid = 'pg_proc'::regclass
        AND dep.objid = p.oid
        AND dep.refclassid = 'pg_extension'::regclass
  );
