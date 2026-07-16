fn main() {
    let closure = |s: &str| -> bool { true };
    let mut options = jsonschema::options();
    options.with_format("custom", closure);
}
