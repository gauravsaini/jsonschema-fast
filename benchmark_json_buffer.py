import json
import timeit
import sys
sys.path.insert(0, "/Users/ektasaini/Desktop/jsonschema-fast")

from jsonschema_fast.validators import Draft7Validator

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

json_bytes = json.dumps(data).encode("utf-8")
validator = Draft7Validator(schema)

# Warmup
validator.validate(data)
validator.validate_json(json_bytes)

t_dict = timeit.timeit("validator.validate(data)", globals=globals(), number=500)
t_bytes = timeit.timeit("validator.validate_json(json_bytes)", globals=globals(), number=500)

print(f"500 Dict Validations: {t_dict:.4f}s ({t_dict/500*1000:.4f} ms/op)")
print(f"500 Raw Bytes FastPath Validations: {t_bytes:.4f}s ({t_bytes/500*1000:.4f} ms/op)")
print(f"FastPath Speedup: {t_dict / t_bytes:.2f}x")
