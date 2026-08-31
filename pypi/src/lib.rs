use pyo3::buffer::PyBuffer;
use pyo3::exceptions::PyBufferError;
use pyo3::prelude::*;

#[pyfunction]
fn search_one(py: Python<'_>, haystack: &Bound<'_, PyAny>, needle: u8) -> PyResult<Option<usize>> {
    let buf = PyBuffer::<u8>::get(haystack)?;
    let slice = buf
        .as_slice(py)
        .ok_or_else(|| PyBufferError::new_err("haystack buffer must be contiguous"))?;

    // SAFETY: ReadOnlyCell<u8> is repr(transparent) over u8
    let bytes: &[u8] = unsafe { &*(slice as *const [_] as *const [u8]) };

    Ok(::ashwa::search_one(bytes, needle))
}

#[pyfunction]
fn search_two(
    py: Python<'_>,
    haystack: &Bound<'_, PyAny>,
    needle: [u8; 2],
) -> PyResult<Option<usize>> {
    let buf = PyBuffer::<u8>::get(haystack)?;
    let slice = buf
        .as_slice(py)
        .ok_or_else(|| PyBufferError::new_err("haystack buffer must be contiguous"))?;

    // SAFETY: ReadOnlyCell<u8> is repr(transparent) over u8
    let bytes: &[u8] = unsafe { &*(slice as *const [_] as *const [u8]) };

    Ok(::ashwa::search_two(bytes, needle))
}

#[pyfunction]
fn search_three(
    py: Python<'_>,
    haystack: &Bound<'_, PyAny>,
    needle: [u8; 3],
) -> PyResult<Option<usize>> {
    let buf = PyBuffer::<u8>::get(haystack)?;
    let slice = buf
        .as_slice(py)
        .ok_or_else(|| PyBufferError::new_err("haystack buffer must be contiguous"))?;

    // SAFETY: ReadOnlyCell<u8> is repr(transparent) over u8
    let bytes: &[u8] = unsafe { &*(slice as *const [_] as *const [u8]) };

    Ok(::ashwa::search_three(bytes, needle))
}

#[pyfunction]
fn search_n(
    py: Python<'_>,
    haystack: &Bound<'_, PyAny>,
    needle: &Bound<'_, PyAny>,
) -> PyResult<Option<usize>> {
    let h_buf = PyBuffer::<u8>::get(haystack)?;
    let h_slice = h_buf
        .as_slice(py)
        .ok_or_else(|| PyBufferError::new_err("haystack buffer must be contiguous"))?;

    // SAFETY: ReadOnlyCell<u8> is repr(transparent) over u8
    let h_bytes: &[u8] = unsafe { &*(h_slice as *const [_] as *const [u8]) };

    if let Ok(n_buf) = PyBuffer::<u8>::get(needle) {
        let n_slice = n_buf
            .as_slice(py)
            .ok_or_else(|| PyBufferError::new_err("needle buffer must be contiguous"))?;
        let n_bytes: &[u8] = unsafe { &*(n_slice as *const [_] as *const [u8]) };
        Ok(::ashwa::search_n(h_bytes, n_bytes))
    } else {
        let n_vec = needle.extract::<Vec<u8>>()?;
        Ok(::ashwa::search_n(h_bytes, &n_vec))
    }
}

#[pymodule]
fn ashwa(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search_one, m)?)?;
    m.add_function(wrap_pyfunction!(search_two, m)?)?;
    m.add_function(wrap_pyfunction!(search_three, m)?)?;
    m.add_function(wrap_pyfunction!(search_n, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
