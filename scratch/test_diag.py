from jsonschema_fast import Draft202012Validator
from jsonschema_fast.validators import _has_unsupported_keywords
schema = {
    "allOf": [
        {"type": "object", "properties": {"foo": {"type": "string"}}},
        {"type": "object", "properties": {"bar": {"type": "integer"}}}
    ]
}
v = Draft202012Validator(schema)
print(f"Bypassed by unsupported keywords? {_has_unsupported_keywords(v._cleaned_schema)}")
print(f"Fallback reasons: {v._fallback_reasons}")
