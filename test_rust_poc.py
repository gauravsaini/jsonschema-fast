from jsonschema_fast import jsonschema_rust as rust_poc
import jsonschema_fast
from jsonschema_fast.exceptions import ValidationError, SchemaError

print("Starting Rust JSONSchema PoC Verification...")

schema = {
    "type": "object",
    "properties": {
        "name": {"type": "string"},
        "age": {"type": "integer", "minimum": 0}
    },
    "required": ["name"]
}

# 1. Test validate() on valid instance
print("\n--- Test 1: Valid Instance ---")
valid_instance = {"name": "Alice", "age": 30}
try:
    rust_poc.validate(valid_instance, schema)
    print("✓ Valid instance passed validation successfully.")
except Exception as e:
    print(f"✗ Unexpected failure: {e}")

# 2. Test validate() on invalid instance
print("\n--- Test 2: Invalid Instance (ValidationError) ---")
invalid_instance = {"name": "Bob", "age": -5}
try:
    rust_poc.validate(invalid_instance, schema)
    print("✗ Expected ValidationError but nothing was raised.")
except ValidationError as e:
    print("✓ Successfully caught jsonschema_fast.exceptions.ValidationError!")
    print(f"  message: {e.message}")
    print(f"  validator: {e.validator}")
    print(f"  path: {list(e.path)}")
    print(f"  schema_path: {list(e.schema_path)}")
    print(f"  instance: {e.instance}")
    print(f"  schema: {e.schema}")
except Exception as e:
    print(f"✗ Caught wrong exception type: {type(e).__name__} - {e}")

# 3. Test validate() on invalid schema
print("\n--- Test 3: Invalid Schema (SchemaError) ---")
invalid_schema = {"type": "invalid_type"}
try:
    rust_poc.validate({"name": "Alice"}, invalid_schema)
    print("✗ Expected SchemaError but nothing was raised.")
except SchemaError as e:
    print("✓ Successfully caught jsonschema_fast.exceptions.SchemaError!")
    print(f"  message: {e.message}")
except Exception as e:
    print(f"✗ Caught wrong exception type: {type(e).__name__} - {e}")

# 4. Test RustValidator class wrapper
print("\n--- Test 4: RustValidator Class ---")
try:
    validator = rust_poc.RustValidator(schema)
    print("✓ Initialized RustValidator successfully.")
    
    print(f"  is_valid(valid_instance): {validator.is_valid(valid_instance)}")
    print(f"  is_valid(invalid_instance): {validator.is_valid(invalid_instance)}")
    
    try:
        validator.validate(invalid_instance)
        print("✗ Expected ValidationError from RustValidator but nothing was raised.")
    except ValidationError as e:
        print("✓ Successfully caught ValidationError from RustValidator!")
        print(f"  message: {e.message}")
        print(f"  path: {list(e.path)}")
except Exception as e:
    print(f"✗ RustValidator test failed: {e}")
