# enum_remove_warn

Like `enum_remove_ignore`, but with `removed_enum_value = "warn"`: the removal is
not emitted into the migration (a warning is printed to stderr, which the test
harness does not capture). `tusker check` still considers the schemas
equivalent.
