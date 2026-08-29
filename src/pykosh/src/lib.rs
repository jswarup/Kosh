use pyo3::prelude::*;

pub mod pybasics;
pub mod pyrube;

/// The Python module definition. The name must match `lib.name` in Cargo.toml.
#[pymodule]
fn pykosh( m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pybasics::add, m)?)?;
    m.add_function(wrap_pyfunction!(pybasics::greet, m)?)?;
    m.add_function(wrap_pyfunction!(pybasics::fibonacci, m)?)?;
    m.add_class::<pybasics::Counter>()?;

    m.add_function(wrap_pyfunction!(pyrube::add_via_rube, m)?)?;

    Ok(())
}
