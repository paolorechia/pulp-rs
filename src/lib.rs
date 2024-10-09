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
