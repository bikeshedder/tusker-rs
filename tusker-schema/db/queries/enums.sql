SELECT
    ns.nspname AS schema,
    t.typname AS name,
    array_agg(e.enumlabel ORDER BY e.enumsortorder) AS labels
FROM pg_catalog.pg_type AS t
JOIN pg_catalog.pg_namespace AS ns ON ns.oid = t.typnamespace
JOIN pg_catalog.pg_enum AS e ON e.enumtypid = t.oid
WHERE ns.nspname = $1
  AND t.typtype = 'e'
GROUP BY ns.nspname, t.typname;
