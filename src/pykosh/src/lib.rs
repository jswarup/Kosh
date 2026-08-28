use pyo3::prelude::*;

/// A simple function: adds two numbers.
#[pyfunction]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Returns a greeting string.
#[pyfunction]
fn greet(name: &str) -> String {
    format!("Hello, {}! This message comes from Rust.", name)
}

/// Computes the nth Fibonacci number (iterative, fast).
#[pyfunction]
fn fibonacci(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let tmp = b;
        b = a + b;
        a = tmp;
    }
    a
}

/// A demo class exposed to Python.
#[pyclass]
struct Counter {
    #[pyo3(get)]
    value: i64,
}

#[pymethods]
impl Counter {
    #[new]
    #[pyo3(signature = (start=None))]
    fn new(start: Option<i64>) -> Self {
        Counter {
            value: start.unwrap_or(0),
        }
    }

    /// Increment the counter by `amount` (default 1) and return the new value.
    #[pyo3(signature = (amount=None))]
    fn increment(&mut self, amount: Option<i64>) -> i64 {
        self.value += amount.unwrap_or(1);
        self.value
    }

    /// Reset the counter to zero.
    fn reset(&mut self) {
        self.value = 0;
    }

    fn __repr__(&self) -> String {
        format!("Counter(value={})", self.value)
    }
}

/// The Python module definition. The name must match `lib.name` in Cargo.toml.
#[pymodule]
fn pykosh( m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    m.add_class::<Counter>()?;
    Ok(())
}
