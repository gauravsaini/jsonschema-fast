import os
os.environ["JSONSCHEMA_FAST_NO_RUST"] = "1"
from jsonschema_fast import Draft202012Validator
schema = {
    "if": {"type": "integer"},
    "then": {"minimum": 10},
    "else": {"maxLength": 3}
}
v = Draft202012Validator(schema)
errors = list(v.iter_errors("abcd"))
for e in errors:
    print(f"Validator: {e.validator}")
    print(f"Message: {e.message}")
    print(f"Context length: {len(e.context) if e.context else 0}")
