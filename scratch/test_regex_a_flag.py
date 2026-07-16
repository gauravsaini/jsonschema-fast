from jsonschema_fast import jsonschema_rust
schema = {
    "type": "string",
    "pattern": "(?a)\\w+"
}
try:
    v = jsonschema_rust.RustValidator(schema, False, None, None)
    print("Rust compilation: SUCCESS")
except Exception as e:
    print(f"Rust compilation: FAILED ({e})")
