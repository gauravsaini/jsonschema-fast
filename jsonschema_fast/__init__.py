"""
An implementation of JSON Schema for Python.

The main functionality is provided by the validator classes for each of the
supported JSON Schema versions.

Most commonly, `jsonschema_fast.validators.validate` is the quickest way to simply
validate a given instance under a schema, and will create a validator
for you.
"""
import warnings as _warnings

# Check for conflicting distributions
try:
    import importlib.metadata as _importlib_metadata
except ImportError:
    try:
        import importlib_metadata as _importlib_metadata  # type: ignore[no-redef]
    except ImportError:
        _importlib_metadata = None  # type: ignore[assignment]

if _importlib_metadata is not None:
    try:
        _dists = {dist.metadata["Name"].lower() for dist in _importlib_metadata.distributions() if dist.metadata and "Name" in dist.metadata}
        if "jsonschema" in _dists and "jsonschema-fast" in _dists:
            _warnings.warn(
                "Both 'jsonschema' and 'jsonschema-fast' distributions are installed. "
                "This can cause import conflicts and unexpected behavior. Please uninstall one of them.",
                UserWarning,
                stacklevel=2,
            )
    except Exception:  # noqa: BLE001, S110
        pass

from jsonschema_fast._format import FormatChecker
from jsonschema_fast._types import TypeChecker
from jsonschema_fast.exceptions import (
    ErrorTree,
    FormatError,
    SchemaError,
    ValidationError,
)
from jsonschema_fast.protocols import Validator
from jsonschema_fast.validators import (
    Draft3Validator,
    Draft4Validator,
    Draft6Validator,
    Draft7Validator,
    Draft201909Validator,
    Draft202012Validator,
    validate,
    validator_for,
)


def __getattr__(name):
    if name == "__version__":
        _warnings.warn(
            "Accessing jsonschema_fast.__version__ is deprecated and will be "
            "removed in a future release. Use importlib.metadata directly "
            "to query for jsonschema_fast's version.",
            DeprecationWarning,
            stacklevel=2,
        )

        from importlib import metadata as _metadata
        for dist_name in ("jsonschema-fast", "jsonschema"):
            try:
                return _metadata.version(dist_name)
            except _metadata.PackageNotFoundError:
                continue
        return "1.0.1"
    elif name == "RefResolver":
        from jsonschema_fast.validators import _RefResolver
        _warnings.warn(
            _RefResolver._DEPRECATION_MESSAGE,
            DeprecationWarning,
            stacklevel=2,
        )
        return _RefResolver
    elif name == "RefResolutionError":
        from jsonschema_fast.exceptions import _RefResolutionError
        _warnings.warn(
            _RefResolutionError._DEPRECATION_MESSAGE,
            DeprecationWarning,
            stacklevel=2,
        )
        return _RefResolutionError

    format_checkers = {
        "draft3_format_checker": Draft3Validator,
        "draft4_format_checker": Draft4Validator,
        "draft6_format_checker": Draft6Validator,
        "draft7_format_checker": Draft7Validator,
        "draft201909_format_checker": Draft201909Validator,
        "draft202012_format_checker": Draft202012Validator,
    }
    ValidatorForFormat = format_checkers.get(name)
    if ValidatorForFormat is not None:
        _warnings.warn(
            f"Accessing jsonschema_fast.{name} is deprecated and will be "
            "removed in a future release. Instead, use the FORMAT_CHECKER "
            "attribute on the corresponding Validator.",
            DeprecationWarning,
            stacklevel=2,
        )
        return ValidatorForFormat.FORMAT_CHECKER

    raise AttributeError(f"module {__name__} has no attribute {name}")


__all__ = [
    "Draft3Validator",
    "Draft4Validator",
    "Draft6Validator",
    "Draft7Validator",
    "Draft201909Validator",
    "Draft202012Validator",
    "ErrorTree",
    "FormatChecker",
    "FormatError",
    "SchemaError",
    "TypeChecker",
    "ValidationError",
    "Validator",
    "validate",
    "validator_for",
]
