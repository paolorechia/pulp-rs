use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn pulp_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<OptimizedClass>()?;
    Ok(())
}

#[pyclass]
struct OptimizedClass {
    value: i32,
}

#[pymethods]
impl OptimizedClass {
    #[new]
    fn new() -> Self {
        OptimizedClass { value: 0 }
    }

    fn set_value(&mut self, value: i32) {
        self.value = value;
    }

    fn get_value(&self) -> i32 {
        self.value
    }
}


use std::collections::HashMap;
use regex::Regex;

#[pyclass]
struct LpElement {
    #[pyo3(get, set)]
    name: Option<String>,
    hash: usize,
    modified: bool,
}

#[pymethods]
impl LpElement {
    #[new]
    fn new(name: Option<String>) -> Self {
        let name = name.map(|n| LpElement::sanitize_name(&n));
        LpElement {
            name,
            hash: std::ptr::addr_of!(name) as usize,
            modified: true,
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.name.clone().unwrap_or_default())
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    fn __hash__(&self) -> PyResult<usize> {
        Ok(self.hash)
    }

    fn __bool__(&self) -> PyResult<bool> {
        Ok(true)
    }
}

impl LpElement {
    fn sanitize_name(name: &str) -> String {
        lazy_static! {
            static ref ILLEGAL_CHARS: Regex = Regex::new(r"[-+\[\] ->/]").unwrap();
        }
        if ILLEGAL_CHARS.is_match(name) {
            println!("Warning: The name {} has illegal characters that will be replaced by _", name);
        }
        ILLEGAL_CHARS.replace_all(name, "_").to_string()
    }
}

use std::collections::HashMap;
use std::iter::FromIterator;
use std::ops::{Add, Sub, Mul, Div, Neg};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use regex::Regex;
use lazy_static::lazy_static;

#[pyclass]
#[derive(Clone)]
struct LpAffineExpression {
    #[pyo3(get, set)]
    constant: f64,
    #[pyo3(get, set)]
    name: Option<String>,
    terms: HashMap<Py<LpElement>, f64>,
}

#[pymethods]
impl LpAffineExpression {
    #[new]
    #[args(e = "None", constant = "0.0", name = "None")]
    fn new(py: Python, e: Option<&PyAny>, constant: f64, name: Option<String>) -> PyResult<Self> {
        let mut expr = LpAffineExpression {
            constant,
            name,
            terms: HashMap::new(),
        };

        if let Some(e) = e {
            if let Ok(other_expr) = e.extract::<PyRef<LpAffineExpression>>() {
                expr.constant = other_expr.constant;
                expr.terms = other_expr.terms.clone();
            } else if let Ok(dict) = e.downcast::<PyDict>() {
                for (k, v) in dict.iter() {
                    let element: Py<LpElement> = k.extract()?;
                    let coeff: f64 = v.extract()?;
                    expr.terms.insert(element, coeff);
                }
            } else if let Ok(list) = e.downcast::<PyList>() {
                for item in list.iter() {
                    let (element, coeff): (Py<LpElement>, f64) = item.extract()?;
                    expr.terms.insert(element, coeff);
                }
            } else if let Ok(element) = e.extract::<PyRef<LpElement>>() {
                expr.terms.insert(element.into_py(py), 1.0);
            } else {
                expr.constant = e.extract()?;
            }
        }

        Ok(expr)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __bool__(&self) -> PyResult<bool> {
        Ok(self.constant != 0.0 || !self.terms.is_empty())
    }

    fn add_term(&mut self, key: Py<LpElement>, value: f64) {
        *self.terms.entry(key).or_insert(0.0) += value;
    }

    fn empty_copy(&self) -> Self {
        LpAffineExpression {
            constant: 0.0,
            name: None,
            terms: HashMap::new(),
        }
    }

    fn copy(&self) -> Self {
        self.clone()
    }

    fn sorted_keys(&self, py: Python) -> PyResult<Vec<Py<LpElement>>> {
        let mut keys: Vec<_> = self.terms.keys().cloned().collect();
        keys.sort_by_key(|k| k.as_ref(py).getattr("name")?.extract::<String>().unwrap_or_default());
        Ok(keys)
    }

    fn __add__(&self, other: &PyAny, py: Python) -> PyResult<Self> {
        let mut result = self.clone();
        result.add_in_place(other, 1.0, py)?;
        Ok(result)
    }

    fn __sub__(&self, other: &PyAny, py: Python) -> PyResult<Self> {
        let mut result = self.clone();
        result.add_in_place(other, -1.0, py)?;
        Ok(result)
    }

    fn __mul__(&self, other: &PyAny, py: Python) -> PyResult<Self> {
        let mut result = self.empty_copy();
        if let Ok(other_expr) = other.extract::<PyRef<LpAffineExpression>>() {
            result.constant = self.constant * other_expr.constant;
            if !other_expr.terms.is_empty() {
                if !self.terms.is_empty() {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        "Non-constant expressions cannot be multiplied",
                    ));
                } else {
                    for (v, x) in &other_expr.terms {
                        result.terms.insert(v.clone(), self.constant * x);
                    }
                }
            } else {
                for (v, x) in &self.terms {
                    result.terms.insert(v.clone(), other_expr.constant * x);
                }
            }
        } else if let Ok(other_float) = other.extract::<f64>() {
            result.constant = self.constant * other_float;
            for (v, x) in &self.terms {
                result.terms.insert(v.clone(), other_float * x);
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported operand type for *",
            ));
        }
        Ok(result)
    }

    fn __truediv__(&self, other: &PyAny, py: Python) -> PyResult<Self> {
        let mut result = self.empty_copy();
        if let Ok(other_expr) = other.extract::<PyRef<LpAffineExpression>>() {
            if !other_expr.terms.is_empty() {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Expressions cannot be divided by a non-constant expression",
                ));
            }
            let divisor = other_expr.constant;
            result.constant = self.constant / divisor;
            for (v, x) in &self.terms {
                result.terms.insert(v.clone(), x / divisor);
            }
        } else if let Ok(other_float) = other.extract::<f64>() {
            result.constant = self.constant / other_float;
            for (v, x) in &self.terms {
                result.terms.insert(v.clone(), x / other_float);
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported operand type for /",
            ));
        }
        Ok(result)
    }

    fn __neg__(&self) -> Self {
        let mut result = self.empty_copy();
        result.constant = -self.constant;
        for (v, x) in &self.terms {
            result.terms.insert(v.clone(), -x);
        }
        result
    }

    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        for (k, v) in &self.terms {
            let k_obj: PyObject = k.as_ref(py).extract()?;
            dict.set_item(k_obj, v)?;
        }
        Ok(dict.into())
    }

    fn is_atomic(&self) -> bool {
        self.terms.len() == 1 && self.constant == 0.0 && self.terms.values().next().unwrap() == &1.0
    }

    fn is_numerical_constant(&self) -> bool {
        self.terms.is_empty()
    }

    fn atom(&self, py: Python) -> PyResult<Option<Py<LpElement>>> {
        if self.is_atomic() {
            Ok(self.terms.keys().next().cloned())
        } else {
            Ok(None)
        }
    }

    fn value(&self, py: Python) -> PyResult<Option<f64>> {
        let mut s = self.constant;
        for (v, x) in &self.terms {
            let var_value = v.as_ref(py).getattr("varValue")?;
            if var_value.is_none() {
                return Ok(None);
            }
            s += var_value.extract::<f64>()? * x;
        }
        Ok(Some(s))
    }

    fn value_or_default(&self, py: Python) -> PyResult<f64> {
        let mut s = self.constant;
        for (v, x) in &self.terms {
            let var_value = v.as_ref(py).call_method0("valueOrDefault")?;
            s += var_value.extract::<f64>()? * x;
        }
        Ok(s)
    }

    fn __le__(&self, other: &PyAny, py: Python) -> PyResult<PyObject> {
        let lhs = self.__sub__(other, py)?;
        py.import("pulp")?.call_method1("LpConstraint", (lhs, "<="))
    }

    fn __ge__(&self, other: &PyAny, py: Python) -> PyResult<PyObject> {
        let lhs = self.__sub__(other, py)?;
        py.import("pulp")?.call_method1("LpConstraint", (lhs, ">="))
    }

    fn __eq__(&self, other: &PyAny, py: Python) -> PyResult<PyObject> {
        let lhs = self.__sub__(other, py)?;
        py.import("pulp")?.call_method1("LpConstraint", (lhs, "=="))
    }

    fn to_dict_list(&self, py: Python) -> PyResult<Vec<PyObject>> {
        let mut result = Vec::new();
        for (k, v) in &self.terms {
            let dict = PyDict::new(py);
            dict.set_item("name", k.as_ref(py).getattr("name")?)?;
            dict.set_item("value", v)?;
            result.push(dict.into());
        }
        Ok(result)
    }
}

impl LpAffineExpression {
    fn add_in_place(&mut self, other: &PyAny, sign: f64, py: Python) -> PyResult<()> {
        if let Ok(other_expr) = other.extract::<PyRef<LpAffineExpression>>() {
            self.constant += other_expr.constant * sign;
            for (v, x) in &other_expr.terms {
                self.add_term(v.clone(), x * sign);
            }
        } else if let Ok(other_float) = other.extract::<f64>() {
            self.constant += other_float * sign;
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Unsupported operand type for addition/subtraction",
            ));
        }
        Ok(())
    }
}

impl ToString for LpAffineExpression {
    fn to_string(&self) -> String {
        let mut terms: Vec<String> = self
            .terms
            .iter()
            .map(|(v, &x)| format!("{}*{}", x, v.to_string()))
            .collect();
        terms.push(self.constant.to_string());
        terms.join(" + ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn test_optimized_class() {
        let mut obj = OptimizedClass::new();
        assert_eq!(obj.get_value(), 0);
        obj.set_value(42);
        assert_eq!(obj.get_value(), 42);
    }

    #[test]
    fn test_lp_element() {
        Python::with_gil(|py| {
            let element = LpElement::new(Some("test_var".to_string()));
            assert_eq!(element.__str__().unwrap(), "test_var");
            assert_eq!(element.__repr__().unwrap(), "test_var");
            assert!(element.__bool__().unwrap());
        });
    }

    #[test]
    fn test_lp_affine_expression() {
        Python::with_gil(|py| {
            let expr = LpAffineExpression::new(py, None, 5.0, None).unwrap();
            assert_eq!(expr.constant, 5.0);
            assert!(expr.terms.is_empty());

            let element = Py::new(py, LpElement::new(Some("x".to_string()))).unwrap();
            let mut expr = LpAffineExpression::new(py, None, 0.0, None).unwrap();
            expr.add_term(element, 2.0);

            assert_eq!(expr.__str__().unwrap(), "2*x + 0");
            assert!(expr.__bool__().unwrap());
            assert!(!expr.is_numerical_constant());
        });
    }

    #[test]
    fn test_lp_affine_expression_operations() {
        Python::with_gil(|py| {
            let expr1 = LpAffineExpression::new(py, None, 5.0, None).unwrap();
            let expr2 = LpAffineExpression::new(py, None, 3.0, None).unwrap();

            let sum = expr1.__add__(&expr2, py).unwrap();
            assert_eq!(sum.constant, 8.0);

            let diff = expr1.__sub__(&expr2, py).unwrap();
            assert_eq!(diff.constant, 2.0);

            let product = expr1.__mul__(&2.0, py).unwrap();
            assert_eq!(product.constant, 10.0);

            let quotient = expr1.__truediv__(&2.0, py).unwrap();
            assert_eq!(quotient.constant, 2.5);

            let negation = expr1.__neg__();
            assert_eq!(negation.constant, -5.0);
        });
    }
}
