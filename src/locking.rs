use std::sync::{Mutex, MutexGuard, RwLock};
use std::marker::{Sync, Send};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::sync::MutexExt;

/// GIL Locked System allowing pyo3 to accept 
/// the unsyncable types
pub struct GIL<T>{
    mt:Mutex<T>
}

impl <T>GIL<T> {
    pub fn new(t: T) -> Self{
        Self { mt: Mutex::new(t) }
    }
    pub fn get(&self) -> PyResult<MutexGuard<'_, T>>{
        Python::with_gil(|py|
            match self.mt.lock_py_attached(py){
                Ok(mut r) => {
                    Ok(r)
                }
                Err(e) => {
                    Err(PyRuntimeError::new_err(e.to_string()))
                }
            }
        )
    }
}

unsafe impl <T> Sync for GIL<T>{}
unsafe impl <T> Send for GIL<T>{}




