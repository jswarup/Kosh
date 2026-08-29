use pyo3::prelude::*;
use kosh::rube::{engine::SimEngine, layout::Layout, adder::Adder};
use kosh::silo::U32;

/// Adds two 32-bit integers by simulating a hardware ripple-carry adder!
#[pyfunction]
pub fn add_via_rube(a: u32, b: u32) -> u32 {
    let mut layout = Layout::New();
    let adder = Adder::<32>::New(&mut layout, "PyAdder32");
    layout.Freeze().expect("Layout compilation failed");

    let mut engine = SimEngine::Create(&layout);

    // Set inputs
    adder.SetA(&mut engine, U32(a));
    adder.SetB(&mut engine, U32(b));

    // Drive the simulation to let signals propagate
    // (N * 3) ticks is plenty for a simple ripple carry
    for _ in 0..(32 * 3) {
        engine.Drive();
    }

    adder.GetSum(&engine) as u32
}

