from jsonschema_fast import jsonschema_rust
schema = {
    "anyOf": [
        {"type": "object", "properties": {"foo": {"type": "string"}}},
        {"type": "object", "properties": {"foo": {"type": "integer"}}}
    ]
}
v = jsonschema_rust.RustValidator(schema, False, None)
errors = v.iter_errors({"foo": 1.5})
print(f"Errors count: {len(errors)}")
for e in errors:
    print(f"Validator: {e.validator}")
    print(f"Message: {e.message}")
    print(f"Path: {list(e.path)}")
    print(f"Schema path: {list(e.schema_path)}")
