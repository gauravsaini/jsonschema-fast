use pyo3::prelude::*;
use pythonize::depythonize;
use jsonschema::Validator as CrateValidator;
use jsonschema::error::ValidationErrorKind;
use serde_json::Value;
use std::str::FromStr;
use rayon::prelude::*;

fn pyany_to_value(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if let Ok(d) = obj.downcast::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::with_capacity(d.len());
        for (k, v) in d.iter() {
            let key: String = if let Ok(s) = k.downcast::<pyo3::types::PyString>() {
                s.to_str()?.to_string()
            } else {
                k.extract()?
            };
            let val = pyany_to_value(py, &v)?;
            map.insert(key, val);
        }
        Ok(Value::Object(map))
    } else if let Ok(l) = obj.downcast::<pyo3::types::PyList>() {
        let mut arr = Vec::with_capacity(l.len());
        for item in l.iter() {
            arr.push(pyany_to_value(py, &item)?);
        }
        Ok(Value::Array(arr))
    } else if let Ok(s) = obj.downcast::<pyo3::types::PyString>() {
        let val = s.to_str()?.to_string();
        Ok(Value::String(val))
    } else if obj.is_none() {
        Ok(Value::Null)
    } else if let Ok(b) = obj.downcast::<pyo3::types::PyBool>() {
        Ok(Value::Bool(b.is_true()))
    } else if let Ok(i) = obj.downcast::<pyo3::types::PyInt>() {
        if let Ok(val) = i.extract::<i64>() {
            Ok(Value::Number(val.into()))
        } else if let Ok(val) = i.extract::<u64>() {
            Ok(Value::Number(val.into()))
        } else {
            let s: String = i.str()?.extract()?;
            if let Ok(n) = serde_json::Number::from_str(&s) {
                Ok(Value::Number(n))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid large int"))
            }
        }
    } else if let Ok(f) = obj.downcast::<pyo3::types::PyFloat>() {
        let val = f.value();
        if let Some(n) = serde_json::Number::from_f64(val) {
            Ok(Value::Number(n))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid float"))
        }
    } else if let Ok(t) = obj.downcast::<pyo3::types::PyTuple>() {
        let mut arr = Vec::with_capacity(t.len());
        for item in t.iter() {
            arr.push(pyany_to_value(py, &item)?);
        }
        Ok(Value::Array(arr))
    } else {
        // Fallback to depythonize for custom types
        match depythonize(obj) {
            Ok(val) => Ok(val),
            Err(e) => Err(e.into()),
        }
    }
}

fn parse_location_to_py(py: Python<'_>, loc: &str) -> PyResult<PyObject> {
    let list = pyo3::types::PyList::empty_bound(py);
    if loc.is_empty() || loc == "/" {
        return Ok(list.into());
    }
    // Location usually starts with '/'
    let segments = loc.split('/');
    for seg in segments {
        if seg.is_empty() {
            continue;
        }
        // Unescape JSON pointer: ~1 becomes /, ~0 becomes ~
        let unescaped = seg.replace("~1", "/").replace("~0", "~");
        if let Ok(idx) = unescaped.parse::<usize>() {
            list.append(idx)?;
        } else {
            list.append(unescaped)?;
        }
    }
    Ok(list.into())
}

fn get_keyword(kind: &ValidationErrorKind) -> &'static str {
    match kind {
        ValidationErrorKind::AdditionalItems { .. } => "additionalItems",
        ValidationErrorKind::AdditionalProperties { .. } => "additionalProperties",
        ValidationErrorKind::AnyOf => "anyOf",
        ValidationErrorKind::BacktrackLimitExceeded { .. } => "pattern",
        ValidationErrorKind::Constant { .. } => "const",
        ValidationErrorKind::Contains => "contains",
        ValidationErrorKind::ContentEncoding { .. } => "contentEncoding",
        ValidationErrorKind::ContentMediaType { .. } => "contentMediaType",
        ValidationErrorKind::Custom { .. } => "custom",
        ValidationErrorKind::Enum { .. } => "enum",
        ValidationErrorKind::ExclusiveMaximum { .. } => "exclusiveMaximum",
        ValidationErrorKind::ExclusiveMinimum { .. } => "exclusiveMinimum",
        ValidationErrorKind::FalseSchema => "false",
        ValidationErrorKind::Format { .. } => "format",
        ValidationErrorKind::FromUtf8 { .. } => "fromUtf8",
        ValidationErrorKind::MaxItems { .. } => "maxItems",
        ValidationErrorKind::Maximum { .. } => "maximum",
        ValidationErrorKind::MaxLength { .. } => "maxLength",
        ValidationErrorKind::MaxProperties { .. } => "maxProperties",
        ValidationErrorKind::MinItems { .. } => "minItems",
        ValidationErrorKind::Minimum { .. } => "minimum",
        ValidationErrorKind::MinLength { .. } => "minLength",
        ValidationErrorKind::MinProperties { .. } => "minProperties",
        ValidationErrorKind::MultipleOf { .. } => "multipleOf",
        ValidationErrorKind::Not { .. } => "not",
        ValidationErrorKind::OneOfMultipleValid => "oneOf",
        ValidationErrorKind::OneOfNotValid => "oneOf",
        ValidationErrorKind::Pattern { .. } => "pattern",
        ValidationErrorKind::PropertyNames { .. } => "propertyNames",
        ValidationErrorKind::Required { .. } => "required",
        ValidationErrorKind::Type { .. } => "type",
        ValidationErrorKind::UnevaluatedItems { .. } => "unevaluatedItems",
        ValidationErrorKind::UnevaluatedProperties { .. } => "unevaluatedProperties",
        ValidationErrorKind::UniqueItems => "uniqueItems",
        ValidationErrorKind::Referencing(..) => "$ref",
    }
}

fn get_validator_value(py: Python<'_>, schema: &Bound<'_, PyAny>, schema_path_list: &Bound<'_, PyAny>) -> PyObject {
    let mut current = schema.clone().into_any();
    if let Ok(list) = schema_path_list.downcast::<pyo3::types::PyList>() {
        for key in list.iter() {
            if let Ok(dict) = current.downcast::<pyo3::types::PyDict>() {
                if let Ok(val) = dict.get_item(&key) {
                    if let Some(val) = val {
                        current = val;
                        continue;
                    }
                }
            }
            if let Ok(lst) = current.downcast::<pyo3::types::PyList>() {
                if let Ok(idx) = key.extract::<usize>() {
                    if let Ok(val) = lst.get_item(idx) {
                        current = val.into_any();
                        continue;
                    }
                }
            }
            return py.None();
        }
    }
    current.unbind()
}

fn get_value_at_path(py: Python<'_>, instance: &Bound<'_, PyAny>, path_list: &Bound<'_, pyo3::types::PyList>) -> PyResult<PyObject> {
    let mut current = instance.clone().into_any();
    for key in path_list.iter() {
        if let Ok(dict) = current.downcast::<pyo3::types::PyDict>() {
            if let Ok(Some(val)) = dict.get_item(&key) {
                current = val;
                continue;
            }
        }
        if let Ok(lst) = current.downcast::<pyo3::types::PyList>() {
            if let Ok(idx) = key.extract::<usize>() {
                if let Ok(val) = lst.get_item(idx) {
                    current = val.into_any();
                    continue;
                }
            }
        }
        if let Ok(tup) = current.downcast::<pyo3::types::PyTuple>() {
            if let Ok(idx) = key.extract::<usize>() {
                if let Ok(val) = tup.get_item(idx) {
                    current = val.into_any();
                    continue;
                }
            }
        }
        return Ok(py.None());
    }
    Ok(current.unbind())
}

use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

fn get_compiled_validator(schema: &Value) -> Result<Arc<CrateValidator>, String> {
    static CACHE: OnceLock<Mutex<HashMap<Value, Arc<CrateValidator>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    
    let mut lock = cache.lock().unwrap();
    if let Some(val) = lock.get(schema) {
        return Ok(Arc::clone(val));
    }
    
    match jsonschema::validator_for(schema) {
        Ok(val) => {
            let val_arc = Arc::new(val);
            lock.insert(schema.clone(), Arc::clone(&val_arc));
            Ok(val_arc)
        }
        Err(e) => Err(e.to_string()),
    }
}

struct ParallelValidatorSpec {
    instance_path: Vec<String>,
    schema_path: Vec<String>,
    items_validator: Arc<CrateValidator>,
}

#[pyclass]
struct RustValidator {
    compiled: Arc<CrateValidator>,
    schema: PyObject,
    specs: Vec<ParallelValidatorSpec>,
}

struct ValidationTask<'a> {
    instance_path: Vec<String>,
    item_val: &'a Value,
    validator: &'a Arc<CrateValidator>,
    schema_path_prefix: &'a [String],
}

struct ParallelError {
    message: String,
    validator: String,
    instance_path: String,
    schema_path: String,
}

fn preprocess_schema(
    schema: &mut Value,
    current_instance_path: &mut Vec<String>,
    current_schema_path: &mut Vec<String>,
    specs: &mut Vec<ParallelValidatorSpec>,
) {
    if let Value::Object(map) = schema {
        // 1. If it's an array type validator with items
        if map.contains_key("items") {
            if let Some(items) = map.get_mut("items") {
                if items.is_object() {
                    let items_schema = items.take();
                    
                    // Replace "items" with true in the parent schema
                    *items = Value::Bool(true);
                    
                    // Compile validator for the items schema
                    if let Ok(val) = get_compiled_validator(&items_schema) {
                        specs.push(ParallelValidatorSpec {
                            instance_path: current_instance_path.clone(),
                            schema_path: {
                                let mut p = current_schema_path.clone();
                                p.push("items".to_string());
                                p
                            },
                            items_validator: val,
                        });
                    }
                }
            }
        }
        
        // 2. Recurse into properties
        if let Some(Value::Object(properties)) = map.get_mut("properties") {
            current_schema_path.push("properties".to_string());
            for (key, subschema) in properties.iter_mut() {
                current_instance_path.push(key.clone());
                current_schema_path.push(key.clone());
                preprocess_schema(subschema, current_instance_path, current_schema_path, specs);
                current_schema_path.pop();
                current_instance_path.pop();
            }
            current_schema_path.pop();
        }
        
        // 3. Recurse into items (tuple validation)
        if let Some(items) = map.get_mut("items") {
            if items.is_array() {
                current_schema_path.push("items".to_string());
                if let Value::Array(arr) = items {
                    for (i, subschema) in arr.iter_mut().enumerate() {
                        current_instance_path.push(i.to_string());
                        current_schema_path.push(i.to_string());
                        preprocess_schema(subschema, current_instance_path, current_schema_path, specs);
                        current_schema_path.pop();
                        current_instance_path.pop();
                    }
                }
                current_schema_path.pop();
            }
        }

        // 4. Recurse into combinators (allOf, anyOf, oneOf)
        for comb in ["allOf", "anyOf", "oneOf"] {
            if let Some(Value::Array(arr)) = map.get_mut(comb) {
                current_schema_path.push(comb.to_string());
                for (i, subschema) in arr.iter_mut().enumerate() {
                    current_schema_path.push(i.to_string());
                    preprocess_schema(subschema, current_instance_path, current_schema_path, specs);
                    current_schema_path.pop();
                }
                current_schema_path.pop();
            }
        }

        // 5. Recurse into conditional schemas
        for cond in ["if", "then", "else"] {
            if let Some(subschema) = map.get_mut(cond) {
                current_schema_path.push(cond.to_string());
                preprocess_schema(subschema, current_instance_path, current_schema_path, specs);
                current_schema_path.pop();
            }
        }

        // 6. Recurse into additionalProperties
        if let Some(additional_properties) = map.get_mut("additionalProperties") {
            if additional_properties.is_object() {
                current_schema_path.push("additionalProperties".to_string());
                current_instance_path.push("*".to_string());
                preprocess_schema(additional_properties, current_instance_path, current_schema_path, specs);
                current_instance_path.pop();
                current_schema_path.pop();
            }
        }
    }
}

fn get_arrays_to_validate<'a>(
    value: &'a Value,
    path: &[String],
    index: usize,
    current_instance_path: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, &'a Vec<Value>)>,
) {
    if index == path.len() {
        if let Value::Array(arr) = value {
            out.push((current_instance_path.clone(), arr));
        }
        return;
    }
    let part = &path[index];
    if part == "*" {
        if let Value::Array(arr) = value {
            for (i, item) in arr.iter().enumerate() {
                current_instance_path.push(i.to_string());
                get_arrays_to_validate(item, path, index + 1, current_instance_path, out);
                current_instance_path.pop();
            }
        } else if let Value::Object(map) = value {
            for (key, val) in map.iter() {
                current_instance_path.push(key.clone());
                get_arrays_to_validate(val, path, index + 1, current_instance_path, out);
                current_instance_path.pop();
            }
        }
    } else {
        if let Value::Object(map) = value {
            if let Some(sub_val) = map.get(part) {
                current_instance_path.push(part.clone());
                get_arrays_to_validate(sub_val, path, index + 1, current_instance_path, out);
                current_instance_path.pop();
            }
        }
    }
}

fn format_path(prefix: &[String], relative: &str) -> String {
    let mut path = String::new();
    for part in prefix {
        path.push('/');
        let escaped = part.replace('~', "~0").replace('/', "~1");
        path.push_str(&escaped);
    }
    if !relative.is_empty() && relative != "/" {
        let trimmed = relative.trim_start_matches('/');
        path.push('/');
        path.push_str(trimmed);
    }
    path
}

fn make_python_validation_error(
    py: Python<'_>,
    err: &ParallelError,
    schema: &Bound<'_, PyAny>,
    root_instance: &Bound<'_, PyAny>,
) -> PyResult<PyObject> {
    let val_error_class = py.import_bound("jsonschema.exceptions")?.getattr("ValidationError")?;
    let path_py = parse_location_to_py(py, &err.instance_path)?;
    let schema_path_py = parse_location_to_py(py, &err.schema_path)?;
    
    let validator_value = get_validator_value(py, schema, schema_path_py.bind(py));
    let sub_instance = get_value_at_path(py, root_instance, path_py.bind(py).downcast::<pyo3::types::PyList>()?)?;

    let kwargs = pyo3::types::PyDict::new_bound(py);
    kwargs.set_item("message", &err.message)?;
    kwargs.set_item("validator", &err.validator)?;
    kwargs.set_item("path", &path_py)?;
    kwargs.set_item("instance", sub_instance)?;
    kwargs.set_item("schema", schema)?;
    kwargs.set_item("schema_path", &schema_path_py)?;
    kwargs.set_item("validator_value", validator_value)?;
    
    let err_obj = val_error_class.call((), Some(&kwargs))?;
    Ok(err_obj.unbind())
}

#[pymethods]
impl RustValidator {
    #[new]
    fn new(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut schema_json: Value = pyany_to_value(py, schema)?;
        
        let schema_str = serde_json::to_string(&schema_json).unwrap_or_default();
        let disable_parallel = schema_str.contains("unevaluatedItems") || schema_str.contains("prefixItems");

        let mut specs = Vec::new();
        let mut current_instance_path = Vec::new();
        let mut current_schema_path = Vec::new();
        
        if !disable_parallel {
            preprocess_schema(
                &mut schema_json,
                &mut current_instance_path,
                &mut current_schema_path,
                &mut specs,
            );
        }
        
        let compiled = get_compiled_validator(&schema_json).map_err(|e| {
            let schema_error_class = py.import_bound("jsonschema.exceptions").unwrap().getattr("SchemaError").unwrap();
            PyErr::from_value_bound(schema_error_class.call1((e,)).unwrap())
        })?;
        
        Ok(RustValidator {
            compiled,
            schema: schema.to_object(py),
            specs,
        })
    }

    fn validate(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<()> {
        let instance_json: Value = pyany_to_value(py, instance)?;
        
        // 1. Prepare tasks
        let mut tasks = Vec::new();
        for spec in &self.specs {
            let mut arrays = Vec::new();
            let mut current_instance_path = Vec::new();
            get_arrays_to_validate(&instance_json, &spec.instance_path, 0, &mut current_instance_path, &mut arrays);
            
            for (arr_path, elements) in arrays {
                for (idx, elem) in elements.iter().enumerate() {
                    tasks.push(ValidationTask {
                        instance_path: {
                            let mut p = arr_path.clone();
                            p.push(idx.to_string());
                            p
                        },
                        item_val: elem,
                        validator: &spec.items_validator,
                        schema_path_prefix: &spec.schema_path,
                    });
                }
            }
        }

        // 2. Run validation (parallel if tasks are many, otherwise sequential)
        let parallel_errors: Vec<ParallelError> = if tasks.len() >= 32 {
            tasks
                .par_iter()
                .flat_map(|task| {
                    let mut errs = Vec::new();
                    for err in task.validator.iter_errors(task.item_val) {
                        let abs_instance_path = format_path(&task.instance_path, err.instance_path.as_str());
                        let abs_schema_path = format_path(task.schema_path_prefix, err.schema_path.as_str());
                        errs.push(ParallelError {
                            message: err.to_string(),
                            validator: get_keyword(&err.kind).to_string(),
                            instance_path: abs_instance_path,
                            schema_path: abs_schema_path,
                        });
                    }
                    errs
                })
                .collect()
        } else {
            let mut errs = Vec::new();
            for task in &tasks {
                for err in task.validator.iter_errors(task.item_val) {
                    let abs_instance_path = format_path(&task.instance_path, err.instance_path.as_str());
                    let abs_schema_path = format_path(task.schema_path_prefix, err.schema_path.as_str());
                    errs.push(ParallelError {
                        message: err.to_string(),
                        validator: get_keyword(&err.kind).to_string(),
                        instance_path: abs_instance_path,
                        schema_path: abs_schema_path,
                    });
                }
            }
            errs
        };

        // 3. Raise first parallel error if any
        if let Some(first_err) = parallel_errors.first() {
            let schema_bound = self.schema.bind(py);
            let err_obj = make_python_validation_error(py, first_err, schema_bound, instance)?;
            return Err(PyErr::from_value_bound(err_obj.into_bound(py)));
        }

        // 4. Validate against the main compiled validator
        let mut errors = self.compiled.iter_errors(&instance_json);
        if let Some(first_error) = errors.next() {
            let val_error_class = py.import_bound("jsonschema.exceptions")?.getattr("ValidationError")?;
            let path_py = parse_location_to_py(py, first_error.instance_path.as_str())?;
            let schema_path_py = parse_location_to_py(py, first_error.schema_path.as_str())?;
            
            let message = first_error.to_string();
            let validator = get_keyword(&first_error.kind);
            let schema_bound = self.schema.bind(py);
            let validator_value = get_validator_value(py, schema_bound, schema_path_py.bind(py));
            let sub_instance = get_value_at_path(py, instance, path_py.bind(py).downcast::<pyo3::types::PyList>()?)?;
            
            let kwargs = pyo3::types::PyDict::new_bound(py);
            kwargs.set_item("message", &message)?;
            kwargs.set_item("validator", validator)?;
            kwargs.set_item("path", &path_py)?;
            kwargs.set_item("instance", sub_instance)?;
            kwargs.set_item("schema", schema_bound)?;
            kwargs.set_item("schema_path", &schema_path_py)?;
            kwargs.set_item("validator_value", validator_value)?;
            
            let err_obj = val_error_class.call((), Some(&kwargs))?;
            return Err(PyErr::from_value_bound(err_obj));
        }
        Ok(())
    }

    fn iter_errors(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<Vec<PyObject>> {
        let instance_json: Value = pyany_to_value(py, instance)?;
        
        // 1. Prepare tasks
        let mut tasks = Vec::new();
        for spec in &self.specs {
            let mut arrays = Vec::new();
            let mut current_instance_path = Vec::new();
            get_arrays_to_validate(&instance_json, &spec.instance_path, 0, &mut current_instance_path, &mut arrays);
            
            for (arr_path, elements) in arrays {
                for (idx, elem) in elements.iter().enumerate() {
                    tasks.push(ValidationTask {
                        instance_path: {
                            let mut p = arr_path.clone();
                            p.push(idx.to_string());
                            p
                        },
                        item_val: elem,
                        validator: &spec.items_validator,
                        schema_path_prefix: &spec.schema_path,
                    });
                }
            }
        }

        // 2. Run validation (parallel if tasks are many, otherwise sequential)
        let parallel_errors: Vec<ParallelError> = if tasks.len() >= 32 {
            tasks
                .par_iter()
                .flat_map(|task| {
                    let mut errs = Vec::new();
                    for err in task.validator.iter_errors(task.item_val) {
                        let abs_instance_path = format_path(&task.instance_path, err.instance_path.as_str());
                        let abs_schema_path = format_path(task.schema_path_prefix, err.schema_path.as_str());
                        errs.push(ParallelError {
                            message: err.to_string(),
                            validator: get_keyword(&err.kind).to_string(),
                            instance_path: abs_instance_path,
                            schema_path: abs_schema_path,
                        });
                    }
                    errs
                })
                .collect()
        } else {
            let mut errs = Vec::new();
            for task in &tasks {
                for err in task.validator.iter_errors(task.item_val) {
                    let abs_instance_path = format_path(&task.instance_path, err.instance_path.as_str());
                    let abs_schema_path = format_path(task.schema_path_prefix, err.schema_path.as_str());
                    errs.push(ParallelError {
                        message: err.to_string(),
                        validator: get_keyword(&err.kind).to_string(),
                        instance_path: abs_instance_path,
                        schema_path: abs_schema_path,
                    });
                }
            }
            errs
        };

        let mut py_errors = Vec::new();
        let schema_bound = self.schema.bind(py);

        // Convert parallel errors
        for err in parallel_errors {
            let err_obj = make_python_validation_error(py, &err, schema_bound, instance)?;
            py_errors.push(err_obj);
        }

        // 3. Run main validation
        for err in self.compiled.iter_errors(&instance_json) {
            let abs_instance_path = err.instance_path.to_string();
            let abs_schema_path = err.schema_path.to_string();
            let p_err = ParallelError {
                message: err.to_string(),
                validator: get_keyword(&err.kind).to_string(),
                instance_path: abs_instance_path,
                schema_path: abs_schema_path,
            };
            let err_obj = make_python_validation_error(py, &p_err, schema_bound, instance)?;
            py_errors.push(err_obj);
        }

        Ok(py_errors)
    }

    fn is_valid(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<bool> {
        let instance_json: Value = pyany_to_value(py, instance)?;
        
        // 1. Prepare tasks
        let mut tasks = Vec::new();
        for spec in &self.specs {
            let mut arrays = Vec::new();
            let mut current_instance_path = Vec::new();
            get_arrays_to_validate(&instance_json, &spec.instance_path, 0, &mut current_instance_path, &mut arrays);
            
            for (arr_path, elements) in arrays {
                for elem in elements {
                    tasks.push(ValidationTask {
                        instance_path: Vec::new(),
                        item_val: elem,
                        validator: &spec.items_validator,
                        schema_path_prefix: &[],
                    });
                }
            }
        }

        // 2. Run elements check (parallel if tasks are many, otherwise sequential)
        let has_error = if tasks.len() >= 32 {
            tasks.par_iter().any(|task| !task.validator.is_valid(task.item_val))
        } else {
            tasks.iter().any(|task| !task.validator.is_valid(task.item_val))
        };
        if has_error {
            return Ok(false);
        }

        // 3. Fallback to main validation check
        Ok(self.compiled.is_valid(&instance_json))
    }
}

#[pyfunction]
fn validate(py: Python<'_>, instance: &Bound<'_, PyAny>, schema: &Bound<'_, PyAny>) -> PyResult<()> {
    let validator = RustValidator::new(py, schema)?;
    validator.validate(py, instance)
}

#[pymodule]
fn jsonschema_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RustValidator>()?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    Ok(())
}
