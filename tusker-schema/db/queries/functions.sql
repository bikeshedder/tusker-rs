SELECT
    ns.nspname AS schema,
    p.proname AS name,
    p.prokind AS kind,
    pg_get_function_identity_arguments(p.oid) AS identity_arguments,
    pg_get_functiondef(p.oid) AS definition
FROM pg_catalog.pg_proc AS p
JOIN pg_catalog.pg_namespace AS ns ON ns.oid = p.pronamespace
WHERE ns.nspname = $1
  AND p.prokind = 'f';
