use	pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "sum_as_string")]
fn	SumAsString( a: usize, b: usize) -> PyResult< String>
{
    return Ok( ( a + b).to_string());
}

#[pymodule]
#[pyo3(name = "kosh_py")]
fn	KoshPy( m: &Bound<'_, PyModule>) -> PyResult< ()>
{
    m.add_function( wrap_pyfunction!( SumAsString, m)?)?;

    return Ok( ());
}
