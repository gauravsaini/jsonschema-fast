import time
import jsonschema
import rust_poc

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

# Compile validators
py_validator = jsonschema.Draft7Validator(schema)
rust_validator = rust_poc.RustValidator(schema)

ITERATIONS = 200

# Benchmark Pure Python (is_valid)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    py_validator.is_valid(large_instance)
t1 = time.perf_counter()
py_time = (t1 - t0) * 1000  # ms
py_ops = ITERATIONS / (t1 - t0)

# Benchmark Rust (is_valid)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    rust_validator.is_valid(large_instance)
t1 = time.perf_counter()
rust_time = (t1 - t0) * 1000  # ms
rust_ops = ITERATIONS / (t1 - t0)

print("=== Large Payload Benchmark (200 iterations) ===")
print(f"Pure Python: {py_time:.2f} ms ({py_ops:.1f} ops/sec)")
print(f"Rust PoC:    {rust_time:.2f} ms ({rust_ops:.1f} ops/sec)")
print(f"Rust is {rust_ops / py_ops:.2f}x faster than Python")
