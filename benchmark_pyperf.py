import pyperf
import sys
import os

# Ensure local jsonschema_fast is imported
sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))
import jsonschema_fast

schema = {
    "type": "object",
    "properties": {
        "users": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "integer", "minimum": 0},
                    "emails": {
                        "type": "array",
                        "items": {"type": "string", "format": "email"}
                    }
                },
                "required": ["name", "age"]
            }
        }
    },
    "required": ["users"]
}

# Create a large instance with 1000 users
large_instance = {
    "users": [
        {
            "name": f"User {i}",
            "age": i % 100,
            "emails": [f"user{i}@example.com", f"alias{i}@domain.com"]
        }
        for i in range(1000)
    ]
}

def add_cmdline_args(cmd, args):
    if args.mode:
        cmd.extend(("--mode", args.mode))

if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.argparser.add_argument(
        "--mode",
        choices=["pure_python", "hybrid", "direct_rust"],
        default="hybrid",
        help="Benchmark mode"
    )
    args = runner.parse_args()
    mode = args.mode

    if mode == "pure_python":
        jsonschema_fast.validators.jsonschema_rust = None
        validator = jsonschema_fast.Draft7Validator(schema)
    elif mode == "hybrid":
        from jsonschema_fast import jsonschema_rust
        validator = jsonschema_fast.Draft7Validator(schema)
    elif mode == "direct_rust":
        from jsonschema_fast import jsonschema_rust
        validator = jsonschema_rust.RustValidator(schema)
    else:
        raise ValueError(f"Unknown mode: {mode}")

    # Warm up the validator
    validator.is_valid(large_instance)

    # Use a constant benchmark name to facilitate pyperf compare_to between runs
    runner.bench_func("jsonschema_validation", validator.is_valid, large_instance)
