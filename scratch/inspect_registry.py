import referencing
from referencing import Registry, Resource

registry = Registry().with_resource(
    "http://example.com/schema",
    Resource.opaque({"type": "integer"})
)
print("Directory of Registry:")
print(dir(registry))
# Let's try iterating or getting internal maps
print("Registry items or keys:")
try:
    for k in registry:
        print(f"Key via iteration: {k}")
except Exception as e:
    print(f"Failed to iterate: {e}")

try:
    # referencing Registry wraps an rpds.HashTrieMap or similar
    # Let's check internal state or properties
    # Let's look at registry._resources or registry._contents?
    # In referencing, registry is actually backed by rpds
    import inspect
    print("rpds trie?")
    for attr in dir(registry):
        if not attr.startswith('_'):
            print(f"{attr}: {type(getattr(registry, attr))}")
except Exception as e:
    print(f"Error inspecting attributes: {e}")
