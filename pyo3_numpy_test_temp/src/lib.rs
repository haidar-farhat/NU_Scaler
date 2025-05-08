use pyo3::prelude::*;
use pyo3::numpy::PyArrayDyn; // Try to use a numpy-specific type

#[pyfunction]
fn get_array_sum(arr: &PyArrayDyn<f64>) -> PyResult<f64> {
    let readonly_arr = arr.as_array();
    Ok(readonly_arr.sum())
}

#[pymodule]
fn pyo3_numpy_test(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_array_sum, m)?)?;
    Ok(())
} 