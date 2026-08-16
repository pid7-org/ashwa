use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyBufferError;
use pyo3::prelude::*;

#[pyfunction]
fn search_one(py: Python<'_>, haystack: &Bound<'_, PyAny>, needle: u8) -> PyResult<Option<usize>> {
    let buf = PyBuffer::<u8>::get(haystack)?;
    let slice = buf
        .as_slice(py)
        .ok_or_else(|| PyBufferError::new_err("haystack buffer must be contiguous"))?;

    // SAFETY: ReadOnlyCell<u8> is repr(transparent) over u8.
    let bytes: &[u8] = unsafe { &*(slice as *const [_] as *const [u8]) };

    Ok(::ashwa::search_one(bytes, needle))
}

#[pymodule]
fn ashwa(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search_one, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
