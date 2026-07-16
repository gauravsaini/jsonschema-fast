import os
os.environ["JSONSCHEMA_FAST_NO_RUST"] = "0"
from jsonschema_fast import Draft202012Validator
from referencing import Registry, Resource

registry = Registry().with_resource(
    "http://nonexistent-local-test12345.com/schema",
    Resource.opaque({"type": "integer"})
)
schema = {"$ref": "http://nonexistent-local-test12345.com/schema"}

v = Draft202012Validator(schema, registry=registry)
errors = list(v.iter_errors("string"))
print(f"Fallback reasons: {getattr(v, '_fallback_reasons', None)}")
print(f"Errors count: {len(errors)}")
for e in errors:
    print(f"Message: {e.message}")
