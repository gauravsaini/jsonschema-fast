from jsonschema_fast import Draft202012Validator
schema = {
    "allOf": [
        {"type": "object", "properties": {"foo": {"type": "string"}}},
        {"type": "object", "properties": {"bar": {"type": "integer"}}}
    ]
}
v = Draft202012Validator(schema)
errors = list(v.iter_errors({"foo": 1, "bar": "a"}))
print(f"Errors count: {len(errors)}")
for e in errors:
    print(f"Validator: {e.validator}")
    print(f"Message: {e.message}")
    print(f"Schema path: {list(e.schema_path)}")
    print(f"Context length: {len(e.context) if e.context else 0}")
    if e.context:
        for c in e.context:
            print(f"  - Context: {c.validator} -> {c.message}")
            print(f"  - Context schema_path: {list(c.schema_path)}")
