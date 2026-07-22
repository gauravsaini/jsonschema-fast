import json
import pytest
from jsonschema_fast import Draft7Validator

def test_is_valid_json_string_and_bytes():
    schema = {
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": ["name", "age"]
    }
    validator = Draft7Validator(schema)
    
    valid_data = {"name": "Alice", "age": 30}
    json_str = json.dumps(valid_data)
    json_bytes = json_str.encode("utf-8")
    
    assert validator.is_valid_json(json_str) is True
    assert validator.is_valid_json(json_bytes) is True
    assert validator.is_valid_json(valid_data) is True
    
    invalid_data = {"name": "Alice", "age": "thirty"}
    invalid_json_str = json.dumps(invalid_data)
    invalid_json_bytes = invalid_json_str.encode("utf-8")
    
    assert validator.is_valid_json(invalid_json_str) is False
    assert validator.is_valid_json(invalid_json_bytes) is False
    assert validator.is_valid_json(invalid_data) is False

def test_validate_json_string_and_bytes():
    schema = {
        "type": "object",
        "properties": {
            "score": {"type": "number"}
        }
    }
    validator = Draft7Validator(schema)
    
    # Valid calls
    validator.validate_json('{"score": 99.5}')
    validator.validate_json(b'{"score": 99.5}')
    
    # Invalid calls should raise ValidationError
    with pytest.raises(Exception):
        validator.validate_json('{"score": "invalid"}')
    with pytest.raises(Exception):
        validator.validate_json(b'{"score": "invalid"}')
