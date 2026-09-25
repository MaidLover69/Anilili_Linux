use crate::error::AppError;
use crate::models::downloads::{
    HEADROOM_BYTES, P1080_BYTES, P360_BYTES, P480_BYTES, P720_BYTES,
};
use pyo3::prelude::*;
use std::fs;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
pub struct StorageCheck {
    #[pyo3(get, set)]
    pub ok: bool,
    #[pyo3(get, set)]
    pub free_bytes: i64,
    #[pyo3(get, set)]
    pub needed_bytes: i64,
}

#[pymethods]
impl StorageCheck {
    #[new]
    pub fn new(ok: bool, free_bytes: i64, needed_bytes: i64) -> Self {
        Self {
            ok,
            free_bytes,
            needed_bytes,
        }
    }
}

pub fn get_download_dir() -> Result<PathBuf, AppError> {
    let app_dir = crate::db::get_app_dir()?;
    let downloads_dir = app_dir.join("downloads");
    fs::create_dir_all(&downloads_dir).map_err(|e| AppError::Other(e.to_string()))?;
    Ok(downloads_dir)
}

pub fn check_storage(quality: &str) -> Result<StorageCheck, AppError> {
    let dir = get_download_dir()?;
    let needed_bytes = match quality {
        "1080p" => P1080_BYTES,
        "720p" => P720_BYTES,
        "480p" => P480_BYTES,
        "360p" => P360_BYTES,
        _ => P720_BYTES, // "best" defaults to 720p/1080p estimate
    };

    // Query free disk space via Rust std::fs or sys_info/statvfs or fallback
    #[cfg(target_family = "unix")]
    let free_bytes = {
        // Simple statvfs via libc

        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let c_path = std::ffi::CString::new(dir.to_str().unwrap_or("/")).unwrap();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                (stat.f_bavail as i64) * (stat.f_frsize as i64)
            } else {
                10_000_000_000i64 // Fallback 10GB
            }
        }
    };

    #[cfg(not(target_family = "unix"))]
    let free_bytes = 10_000_000_000i64;

    let ok = free_bytes >= (needed_bytes + HEADROOM_BYTES);
    Ok(StorageCheck {
        ok,
        free_bytes,
        needed_bytes,
    })
}

#[pyfunction]
pub fn check_storage_sync(quality: String) -> PyResult<StorageCheck> {
    let check = check_storage(&quality).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
    Ok(check)
}

#[pyfunction]
pub fn check_storage_for_download(quality: String, callback: PyObject) -> PyResult<()> {

    let res = check_storage(&quality);
    Python::with_gil(|py| match res {
        Ok(check) => {
            let none_err: Option<String> = None;
            let _ = callback.call1(py, (true, check, none_err));
        }
        Err(err) => {
            let err_str = Some(err.to_string());
            let dummy = StorageCheck::new(false, 0, 0);
            let _ = callback.call1(py, (false, dummy, err_str));
        }
    });
    Ok(())
}

#[pyfunction]
pub fn get_download_directory(callback: PyObject) -> PyResult<()> {
    let res = get_download_dir();
    Python::with_gil(|py| match res {
        Ok(dir) => {
            let dir_str = dir.to_string_lossy().to_string();
            let none_err: Option<String> = None;
            let _ = callback.call1(py, (true, dir_str, none_err));
        }
        Err(err) => {
            let err_str = Some(err.to_string());
            let _ = callback.call1(py, (false, "", err_str));
        }
    });
    Ok(())
}
