import sys
import importlib

# Register the meta path shim finder so that all submodules (e.g. jsonschema.validators)
# automatically resolve to the corresponding jsonschema_fast modules.
class _ShimFinder:
    def find_spec(self, fullname, path, target=None):
        if fullname == "jsonschema" or fullname.startswith("jsonschema."):
            fast_name = "jsonschema_fast" + fullname[len("jsonschema"):]
            try:
                mod = importlib.import_module(fast_name)
                sys.modules[fullname] = mod
                return mod.__spec__
            except ModuleNotFoundError:
                return None
        return None

sys.meta_path.insert(0, _ShimFinder())

# Expose all top-level public APIs from jsonschema_fast
from jsonschema_fast import *
