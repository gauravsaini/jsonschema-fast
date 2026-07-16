from referencing import Resource
r = Resource.opaque({"type": "integer"})
print(dir(r))
print(f"contents: {r.contents}")
