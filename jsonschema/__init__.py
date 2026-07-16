import sys
import importlib

# Register the meta path shim finder so that all submodules (e.g. jsonschema.validators)
# automatically resolve to the corresponding jsonschema_fast modules.
class _DummyLoader:
    def create_module(self, spec):
        return sys.modules[spec.name]
    def exec_module(self, module):
        pass

class _ShimFinder:
    def find_spec(self, fullname, path, target=None):
        if fullname == "jsonschema" or fullname.startswith("jsonschema."):
            fast_name = "jsonschema_fast" + fullname[len("jsonschema"):]
            try:
                mod = importlib.import_module(fast_name)
                sys.modules[fullname] = mod
                
                from importlib.machinery import ModuleSpec
                is_package = hasattr(mod, "__path__")
                spec = ModuleSpec(fullname, _DummyLoader(), is_package=is_package)
                if is_package:
                    spec.submodule_search_locations = list(mod.__path__)
                return spec
            except ModuleNotFoundError:
                return None
        return None

sys.meta_path.insert(0, _ShimFinder())

# Expose all top-level public APIs from jsonschema_fast
from jsonschema_fast import *
