"""
Some (initially private) typing helpers for jsonschema_fast's types.
"""
from collections.abc import Callable, Iterable
from typing import Any, Protocol

import referencing.jsonschema

from jsonschema_fast.protocols import Validator


class SchemaKeywordValidator(Protocol):
    def __call__(
        self,
        validator: Validator,
        value: Any,
        instance: Any,
        schema: referencing.jsonschema.Schema,
    ) -> None:
        ...


id_of = Callable[[referencing.jsonschema.Schema], str | None]


ApplicableValidators = Callable[
    [referencing.jsonschema.Schema],
    Iterable[tuple[str, Any]],
]
