from jsonschema_fast import Draft202012Validator
schema = {
    "type": "string",
    "pattern": "^test\\Z"
}
v = Draft202012Validator(schema)
errors = list(v.iter_errors("test"))
print(f"Errors for 'test': {len(errors)}")
errors2 = list(v.iter_errors("test\n"))
print(f"Errors for 'test\\n': {len(errors2)}")
print(f"Fallback reasons: {getattr(v, '_fallback_reasons', None)}")
