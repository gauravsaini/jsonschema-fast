use jsonschema::Validator;
use serde_json::json;

fn main() {
    let schema = json!({
        "anyOf": [
            {"type": "string"},
            {"type": "number"}
        ]
    });
    let instance = json!(true);
    let compiled = jsonschema::options().build(&schema).unwrap();
    let mut errors = compiled.iter_errors(&instance);
    if let Some(err) = errors.next() {
        println!("{:?}", err);
    }
}
