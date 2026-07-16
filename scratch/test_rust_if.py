from jsonschema_fast import jsonschema_rust
schema = {
    "if": {"type": "integer"},
    "then": {"minimum": 10},
    "else": {"maxLength": 3}
}
v = jsonschema_rust.RustValidator(schema, False, None)
errors = v.iter_errors("abcd")
for e in errors:
    print(f"Validator: {e.validator}")
    print(f"Message: {e.message}")
    print(f"Path: {list(e.path)}")
    print(f"Schema path: {list(e.schema_path)}")
