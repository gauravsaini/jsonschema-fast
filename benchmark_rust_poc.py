import time
import jsonschema
import rust_poc

schema = {
    "type": "object",
    "properties": {
        "user": {
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
    },
    "required": ["user"]
}

valid_instance = {
    "user": {
        "name": "Saurabh",
        "age": 30,
        "emails": ["saurabh@example.com", "test@domain.com"]
    }
}

invalid_instance = {
    "user": {
        "name": "Saurabh",
        "age": -1,
        "emails": ["invalid-email"]
    }
}

# 1. Warm up & Verify
jsonschema.validate(valid_instance, schema)
rust_poc.validate(valid_instance, schema)

# Compile validators
py_validator = jsonschema.Draft7Validator(schema)
rust_validator = rust_poc.RustValidator(schema)

ITERATIONS = 20000

# Benchmark Pure Python (is_valid)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    py_validator.is_valid(valid_instance)
t1 = time.perf_counter()
py_time = (t1 - t0) * 1000  # ms
py_ops = ITERATIONS / (t1 - t0)

# Benchmark Rust (is_valid)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    rust_validator.is_valid(valid_instance)
t1 = time.perf_counter()
rust_time = (t1 - t0) * 1000  # ms
rust_ops = ITERATIONS / (t1 - t0)

print("=== Benchmark results (is_valid) ===")
print(f"Iterations: {ITERATIONS}")
print(f"Pure Python: {py_time:.2f} ms ({py_ops:.1f} ops/sec)")
print(f"Rust PoC:    {rust_time:.2f} ms ({rust_ops:.1f} ops/sec)")
print(f"Speedup:     {rust_ops / py_ops:.2f}x" if rust_ops > py_ops else f"Speedup:     {py_ops / rust_ops:.2f}x")

# Benchmark Pure Python (validate - valid path)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    py_validator.validate(valid_instance)
t1 = time.perf_counter()
py_val_time = (t1 - t0) * 1000  # ms
py_val_ops = ITERATIONS / (t1 - t0)

# Benchmark Rust (validate - valid path)
t0 = time.perf_counter()
for _ in range(ITERATIONS):
    rust_validator.validate(valid_instance)
t1 = time.perf_counter()
rust_val_time = (t1 - t0) * 1000  # ms
rust_val_ops = ITERATIONS / (t1 - t0)

print("\n=== Benchmark results (validate - valid path) ===")
print(f"Pure Python: {py_val_time:.2f} ms ({py_val_ops:.1f} ops/sec)")
print(f"Rust PoC:    {rust_val_time:.2f} ms ({rust_val_ops:.1f} ops/sec)")
print(f"Speedup:     {rust_val_ops / py_val_ops:.2f}x")
