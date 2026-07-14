import pyperf
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.dirname(__file__)))
import jsonschema

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

# Create an invalid instance where one user has negative age (minimum: 0)
invalid_instance = {
    "users": [
        {
            "name": f"User {i}",
            "age": i % 100 if i != 500 else -5,  # 500th user is invalid
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
        choices=["pure_python", "hybrid"],
        default="hybrid",
        help="Benchmark mode"
    )
    args = runner.parse_args()
    mode = args.mode

    if mode == "pure_python":
        jsonschema.validators.jsonschema_rust = None
        validator = jsonschema.Draft7Validator(schema)
    elif mode == "hybrid":
        import jsonschema_rust
        validator = jsonschema.Draft7Validator(schema)
    else:
        raise ValueError(f"Unknown mode: {mode}")

    # Function to benchmark: we retrieve all errors (which is what validate/iter_errors does)
    def run_validation():
        list(validator.iter_errors(invalid_instance))

    # Warm up
    run_validation()

    runner.bench_func("jsonschema_invalid_validation", run_validation)
