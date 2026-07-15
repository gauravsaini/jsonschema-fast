"""
A performance benchmark using the example from issue #232.

See https://github.com/python-jsonschema_fast/jsonschema_fast/pull/232.
"""
from pathlib import Path

from pyperf import Runner
from referencing import Registry

from jsonschema_fast.tests._suite import Version
import jsonschema_fast

issue232 = Version(
    path=Path(__file__).parent / "issue232",
    remotes=Registry(),
    name="issue232",
)


if __name__ == "__main__":
    issue232.benchmark(
        runner=Runner(),
        Validator=jsonschema_fast.Draft4Validator,
    )
