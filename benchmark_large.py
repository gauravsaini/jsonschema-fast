import time
import jsonschema_fast
from jsonschema_fast import jsonschema_rust

schema = {
    "type": "object",
    "properties": {
        "users": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "integer", "minimum": 0},
                    "emails": {
                        "type": "array",
                        "items": {"type": "string", "format": "email"}
                    }
                },
                "required": ["name", "age"]
            }
        }
    },
    "required": ["users"]
}

# Create a large instance with 1000 users
large_instance = {
    "users": [
        {
            "name": f"User {i}",
            "age": i % 100,
            "emails": [f"user{i}@example.com", f"alias{i}@domain.com"]
        }
        for i in range(1000)
    ]
}

ITERATIONS = 200

# 1. Benchmark Hybrid Validator (jsonschema_fast with Rust enabled)
py_hybrid_validator = jsonschema_fast.Draft7Validator(schema)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    py_hybrid_validator.is_valid(large_instance)
t1 = time.perf_counter()
hybrid_time = (t1 - t0) * 1000  # ms
hybrid_ops = ITERATIONS / (t1 - t0)

# 2. Benchmark Direct Rust Validator (jsonschema_rust.RustValidator)
rust_validator = jsonschema_rust.RustValidator(schema)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    rust_validator.is_valid(large_instance)
t1 = time.perf_counter()
rust_time = (t1 - t0) * 1000  # ms
rust_ops = ITERATIONS / (t1 - t0)

# 3. Benchmark Pure Python (jsonschema_fast with Rust disabled)
jsonschema_fast.validators.jsonschema_rust = None
pure_py_validator = jsonschema_fast.Draft7Validator(schema)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    pure_py_validator.is_valid(large_instance)
t1 = time.perf_counter()
pure_py_time = (t1 - t0) * 1000  # ms
pure_py_ops = ITERATIONS / (t1 - t0)

print("=== Large Payload Benchmark (200 iterations) ===")
print(f"Pure Python:       {pure_py_time:.2f} ms ({pure_py_ops:.1f} ops/sec)")
print(f"Hybrid (Rust):     {hybrid_time:.2f} ms ({hybrid_ops:.1f} ops/sec)")
print(f"Direct Rust C-Ext: {rust_time:.2f} ms ({rust_ops:.1f} ops/sec)")
print(f"Hybrid Speedup:    {hybrid_ops / pure_py_ops:.2f}x faster")
print(f"Direct C-Ext Speedup: {rust_ops / pure_py_ops:.2f}x faster")
