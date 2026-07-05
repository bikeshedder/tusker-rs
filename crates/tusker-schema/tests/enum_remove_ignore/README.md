# enum_remove_ignore

With `removed_enum_value = "ignore"`, a removed enum value is not actionable:
`tusker diff` emits nothing for it and `tusker check` considers the schemas
equivalent. Adding a value is still a safe, actionable change, so the reverse
migration re-adds it with `ADD VALUE`.
