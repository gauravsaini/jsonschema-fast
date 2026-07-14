import timeit
import sys

# Ensure we import the local jsonschema
sys.path.insert(0, "/home/gsai/.gemini/antigravity-cli/scratch/jsonschema")
import jsonschema
from jsonschema.validators import Draft7Validator

# Complex schema with items and properties
schema = {
    "type": "object",
    "properties": {
        "users": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "id": {"type": "integer"},
                    "name": {"type": "string"},
                    "email": {"type": "string"},
                    "roles": {
                        "type": "array",
                        "items": {"type": "string"}
                    }
                },
                "required": ["id", "name"]
            }
        }
    }
}

# Create a data instance
data = {
    "users": [
        {
            "id": i,
            "name": f"User {i}",
            "email": f"user{i}@example.com",
            "roles": ["admin", "user"]
        }
        for i in range(500)
    ]
}

validator = Draft7Validator(schema)

# Warm up
validator.validate(data)

# Run benchmark
t_validate = timeit.timeit("validator.validate(data)", globals=globals(), number=200)
print(f"Validation time (200 runs): {t_validate:.4f} seconds")
print(f"Time per validation: {t_validate / 200 * 1000:.4f} ms")
