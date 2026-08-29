use pyo3::prelude::*;

/// A simple function: adds two numbers.
#[pyfunction]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Returns a greeting string.
#[pyfunction]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! This message comes from Rust.", name)
}

/// Computes the nth Fibonacci number (iterative, fast).
#[pyfunction]
pub fn fibonacci(n: u64) -> u64 {
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
pub struct Counter {
    #[pyo3(get)]
    pub value: i64,
}

#[pymethods]
impl Counter {
    #[new]
    #[pyo3(signature = (start=None))]
    pub fn new(start: Option<i64>) -> Self {
        Counter {
            value: start.unwrap_or(0),
        }
    }

    /// Increment the counter by `amount` (default 1) and return the new value.
    #[pyo3(signature = (amount=None))]
    pub fn increment(&mut self, amount: Option<i64>) -> i64 {
        self.value += amount.unwrap_or(1);
        self.value
    }

    /// Reset the counter to zero.
    pub fn reset(&mut self) {
        self.value = 0;
    }

    pub fn __repr__(&self) -> String {
        format!("Counter(value={})", self.value)
    }
}

