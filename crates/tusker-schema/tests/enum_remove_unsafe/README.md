# enum_remove_unsafe

Removing a value from an enum is unsafe (PostgreSQL cannot drop an enum label).
With the default `removed_enum_value = "unsafe"`, `tusker diff` emits a guarded
migration (`RAISE EXCEPTION`) so the removal is surfaced and `tusker check`
reports a difference. The reverse migration re-adds the value with a plain
`ADD VALUE`.
