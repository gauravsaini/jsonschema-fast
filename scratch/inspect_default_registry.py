from jsonschema_fast import Draft202012Validator
v = Draft202012Validator({})
print(f"Registry keys count: {len(v._registry)}")
print("First 5 keys:")
print(list(v._registry.keys())[:5])
