use pyo3::prelude::*;
use pythonize::depythonize;
use jsonschema::Validator as CrateValidator;
use jsonschema::error::ValidationErrorKind;
use serde_json::Value;
use std::str::FromStr;

fn pyany_to_value(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<Value> {
    if obj.is_none() {
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
    } else if let Ok(s) = obj.downcast::<pyo3::types::PyString>() {
        let val: String = s.extract()?;
        Ok(Value::String(val))
    } else if let Ok(d) = obj.downcast::<pyo3::types::PyDict>() {
        let mut map = serde_json::Map::with_capacity(d.len());
        for (k, v) in d.iter() {
            let key: String = k.extract()?;
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

#[pyclass]
struct RustValidationError {
    #[pyo3(get)]
    message: String,
    #[pyo3(get)]
    validator: String,
    #[pyo3(get)]
    path: PyObject,
    #[pyo3(get)]
    schema_path: PyObject,
    #[pyo3(get)]
    validator_value: PyObject,
}

#[pyclass]
struct RustValidator {
    compiled: CrateValidator,
    schema: PyObject,
}

#[pymethods]
impl RustValidator {
    #[new]
    fn new(py: Python<'_>, schema: &Bound<'_, PyAny>) -> PyResult<Self> {
        let schema_json: Value = pyany_to_value(py, schema)?;
        let compiled = jsonschema::validator_for(&schema_json).map_err(|e| {
            let schema_error_class = py.import_bound("jsonschema.exceptions").unwrap().getattr("SchemaError").unwrap();
            PyErr::from_value_bound(schema_error_class.call1((e.to_string(),)).unwrap())
        })?;
        Ok(RustValidator {
            compiled,
            schema: schema.to_object(py),
        })
    }

    fn validate(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<()> {
        let instance_json: Value = pyany_to_value(py, instance)?;
        let mut errors = self.compiled.iter_errors(&instance_json);
        if let Some(first_error) = errors.next() {
            let val_error_class = py.import_bound("jsonschema.exceptions")?.getattr("ValidationError")?;
            let path_py = parse_location_to_py(py, first_error.instance_path.as_str())?;
            let schema_path_py = parse_location_to_py(py, first_error.schema_path.as_str())?;
            
            let message = first_error.to_string();
            let validator = get_keyword(&first_error.kind);
            let schema_bound = self.schema.bind(py);
            let validator_value = get_validator_value(py, schema_bound, schema_path_py.bind(py));
            
            let kwargs = pyo3::types::PyDict::new_bound(py);
            kwargs.set_item("message", &message)?;
            kwargs.set_item("validator", validator)?;
            kwargs.set_item("path", &path_py)?;
            kwargs.set_item("instance", instance)?;
            kwargs.set_item("schema", schema_bound)?;
            kwargs.set_item("schema_path", &schema_path_py)?;
            kwargs.set_item("validator_value", validator_value)?;
            
            let err_obj = val_error_class.call((), Some(&kwargs))?;
            return Err(PyErr::from_value_bound(err_obj));
        }
        Ok(())
    }

    fn iter_errors(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<Vec<RustValidationError>> {
        let instance_json: Value = pyany_to_value(py, instance)?;
        let errors = self.compiled.iter_errors(&instance_json);
        let mut py_errors = Vec::new();
        
        for err in errors {
            let path_py = parse_location_to_py(py, err.instance_path.as_str())?;
            let schema_path_py = parse_location_to_py(py, err.schema_path.as_str())?;
            
            let message = err.to_string();
            let validator = get_keyword(&err.kind).to_string();
            let schema_bound = self.schema.bind(py);
            let validator_value = get_validator_value(py, schema_bound, schema_path_py.bind(py));
            
            py_errors.push(RustValidationError {
                message,
                validator,
                path: path_py,
                schema_path: schema_path_py,
                validator_value,
            });
        }
        
        Ok(py_errors)
    }

    fn is_valid(&self, py: Python<'_>, instance: &Bound<'_, PyAny>) -> PyResult<bool> {
        let instance_json: Value = pyany_to_value(py, instance)?;
        Ok(self.compiled.is_valid(&instance_json))
    }
}

#[pyfunction]
fn validate(py: Python<'_>, instance: &Bound<'_, PyAny>, schema: &Bound<'_, PyAny>) -> PyResult<()> {
    let schema_json: Value = pyany_to_value(py, schema)?;
    let instance_json: Value = pyany_to_value(py, instance)?;

    let compiled = match jsonschema::validator_for(&schema_json) {
        Ok(c) => c,
        Err(e) => {
            let schema_error_class = py.import_bound("jsonschema.exceptions")?.getattr("SchemaError")?;
            let err_msg = e.to_string();
            let err_obj = schema_error_class.call1((err_msg,))?;
            return Err(PyErr::from_value_bound(err_obj));
        }
    };

    let mut errors = compiled.iter_errors(&instance_json);
    if let Some(first_error) = errors.next() {
        let val_error_class = py.import_bound("jsonschema.exceptions")?.getattr("ValidationError")?;
        let path_py = parse_location_to_py(py, first_error.instance_path.as_str())?;
        let schema_path_py = parse_location_to_py(py, first_error.schema_path.as_str())?;
        
        let message = first_error.to_string();
        let validator = get_keyword(&first_error.kind);
        let validator_value = get_validator_value(py, schema, schema_path_py.bind(py));
        
        let kwargs = pyo3::types::PyDict::new_bound(py);
        kwargs.set_item("message", &message)?;
        kwargs.set_item("validator", validator)?;
        kwargs.set_item("path", &path_py)?;
        kwargs.set_item("instance", instance)?;
        kwargs.set_item("schema", schema)?;
        kwargs.set_item("schema_path", &schema_path_py)?;
        kwargs.set_item("validator_value", validator_value)?;
        
        let err_obj = val_error_class.call((), Some(&kwargs))?;
        return Err(PyErr::from_value_bound(err_obj));
    }

    Ok(())
}

#[pymodule]
fn jsonschema_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RustValidator>()?;
    m.add_class::<RustValidationError>()?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    Ok(())
}
