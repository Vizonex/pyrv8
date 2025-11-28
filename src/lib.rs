use std::{time::Duration, task::Poll};

use pyo3::{exceptions::{PyKeyError, PyNotADirectoryError, PyRuntimeError, PyValueError}, prelude::*, types::PyTuple};
use rustyscript::{Error as RSError, Runtime, RuntimeOptions, deno_core::PollEventLoopOptions};
use rustyscript::js_value::Promise;
use serde_pyobject::{to_pyobject, from_pyobject};

pub mod locking;
use locking::GIL;


#[pyclass]
struct Context{
    runtime:GIL<Runtime>
}




/// Used multiple times throughout the code this is used to get rid of the annoyance
/// of attaching the gil or detaching it when done.
#[inline]
pub fn serde_to_python(value: serde_json::Value) -> PyResult<Py<PyAny>>{
    Python::with_gil( |py|
        match to_pyobject(py, &value) {
            Ok(res) => {Ok(res.unbind())}
            Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
        }
    )
}


/// Shortcut for creating runtime variables
#[inline]
pub fn create_runtime(timeout:Option<f64>, max_heap_size: Option<usize>) -> PyResult<GIL<Runtime>>{
    let mut options = RuntimeOptions::default();
    if let Some(timeout) = timeout{
        options.timeout = Duration::from_secs_f64(timeout);
    }
    options.max_heap_size = max_heap_size;
    match Runtime::new(options){
        Ok(runtime) => {Ok(GIL::new(runtime))},
        Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
    }
}

#[inline]
pub fn python_args_to_serde(
    py_args: &Bound<'_, PyTuple>,
) -> PyResult<Vec<serde_json::Value>>{
    if py_args.len() < 1 {
        return Ok(Vec::new());
    }
    let mut s = Vec::new();

    // My Logic on rust may not be as clean as someone else's 
    // if you think you can do better than me, make me a pull request - Vizonex
    for a in py_args.iter().map(|a|from_pyobject(a)){
        match a {
            Ok(r) => {s.push(r);}
            Err(e) => {return Err(PyValueError::new_err(e.to_string()))}
        }
    }
    return Ok(s);
}



/// Inspired by asyncio.Future
/// this is a Lower level version of Promise type in pyrv8
/// the upper level called Promise can do more asyncio-like things
/// and can the upper version can inherit the Runtime as a parent.
#[pyclass] 
struct JSPromise {
    fut: GIL<Promise<serde_json::Value>>,
    result: Option<PyResult<Py<PyAny>>>,
    done: bool
}

impl JSPromise {
    /// Private static method in rust to attach a Promise to a python
    /// class object
    pub fn wrap(fut: Promise<serde_json::Value>) -> Self{
        Self {fut: GIL::new(fut), done:false, result:None}
    }
}

#[pymethods]
impl JSPromise {
    
    /// Returns true if Exception was thrown or a Result came back
    /// from walking through the eventloop
    #[getter]
    pub fn is_done(&self) -> bool {
        self.done
    }

    // Steps a single increment into the eventloop while also checking if the value
    // is polled. it can't run poll_promise directly since we covered runtime in a 
    // mutex to prevent pyo3 from disallowing it to exist.
    // pub fn step(&mut self, ctx:&mut Context) -> PyResult<bool> {
    //     // ctx.advance(None, None)?;
    //     // let fut = self.fut.get()?;
    //     // if ctx.runtime.get()?.block_on(move |runtime| async move { fut.resolve(runtime.deno_runtime()).await }){
    //     // }
    // }
}

// /// An Already loaded version of a Js Module Handle...
// #[pyclass]
// struct JsHandle {
//     pub module: GIL<ModuleHandle>,
// }

// /// An Unloaded version of a Js Module or ready to be prepared...
// #[pyclass]
// struct JsModule {
//     module: GIL<Module>
// }

// #[pymethods]
// impl JsModule {
//     #[new]
//     pub fn new(filename:String, contents: String) -> Self {
//         Self{module:GIL::new(Module::new(filename, contents))}
//     }

//     #[staticmethod]
//     pub fn load(filename:String) -> PyResult<Self>{
//         match Module::load(filename){   
//             Ok(x) => {Ok(Self{module:GIL::new(x)})}
//             Err(e) => {Err(PyFileNotFoundError::new_err(e.to_string()))}
//         }
//     }

//     #[staticmethod]
//     pub fn load_dir(directory:String) -> PyResult<Vec<Self>>{
//         // Mirrors load_dir from Module::load_dir but for our python-made class object...
//         let mut files: Vec<Self> = Vec::new();
//         for file in read_dir(directory)? {
//             let file = file?;
//             if let Some(filename) = file.path().to_str() {
//                 // Skip non-js files
//                 let extension = Path::new(&filename)
//                     .extension()
//                     .and_then(OsStr::to_str)
//                     .unwrap_or_default();
//                 if !["js", "ts"].contains(&extension) {
//                     continue;
//                 }

//                 files.push(Self::load(filename.to_string())?);
//             }
//         }
//         Ok(files)
//     }

//     #[getter]
//     pub fn filename(&self) -> PyResult<String>{
//         Ok(self.module.get()?.filename().to_string_lossy().to_string())
//     }

//     #[getter]
//     pub fn contents(&self) -> PyResult<String>{
//         Ok(self.module.get()?.contents().to_string())
//     }
// }

// impl JsHandle {
//     pub fn new(handle:ModuleHandle) -> Self {
//         Self{module:GIL::new(handle)}
//     }
// }


// #[pymethods]
// impl JsHandle {
//     #[getter]
//     pub fn filename(&self) -> PyResult<String>{
//         Ok(self.module.get()?.module().filename().to_string_lossy().to_string())
//     }

//     #[getter]
//     pub fn contents(&self) -> PyResult<String>{
//         Ok(self.module.get()?.module().contents().to_string())
//     }
    
// }

#[pymethods]
impl Context {
    #[new]
    #[pyo3(signature = (timeout=None, max_heap_size=None))]
    pub fn new(timeout:Option<f64>, max_heap_size: Option<usize>) -> PyResult<Self> {
        Ok(Self{runtime: create_runtime(timeout, max_heap_size)?})
    }
    #[getter]
    pub fn timeout(&self) -> PyResult<f64> {
        Ok(self.runtime.get()?.timeout().as_secs_f64())
    }

    #[getter]
    pub fn current_dir(&self) -> PyResult<String> {
        Ok(self.runtime.get()?.current_dir().to_string_lossy().to_string())
    }

    pub fn set_current_dir(&mut self, path: String) -> PyResult<()>{
        match self.runtime.get()?.set_current_dir(path) {
            Ok(_) => {Ok(())}
            Err(e) => {Err(PyNotADirectoryError::new_err(e.to_string()))}
        }
    }

    // Still being worked on...
    // /// Advances eventloop by a single tick this best used 
    // /// with trio or anyio
    // pub async fn advance_async(&mut self, 
    //     wait_for_inspector: Option<bool>,
    //     pump_v8_message_loop: Option<bool>,
    // ) -> PyResult<bool> {
    //     let mut options= PollEventLoopOptions::default();
    //     if let Some(wait_for_inspector) = wait_for_inspector{
    //         options.wait_for_inspector = wait_for_inspector
    //     }
    //     if let Some(pump_v8_message_loop) = pump_v8_message_loop {
    //         options.pump_v8_message_loop = pump_v8_message_loop;
    //     }
        
    //     match self.runtime.get()?.advance_event_loop_async(options).await {
    //         Ok(b) => {Ok(b)},
    //         Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
    //     }
    // }

    /// Advances eventloop by a single tick this best used 
    /// with python asyncio, uvloop, winloop or rloop.
    /// This is meant to be used with Javascript Promise Values since 
    /// an asyncio eventloop can call this if it's waiting on a Promise value
    #[pyo3(signature = (wait_for_inspector=None, pump_v8_message_loop=None))]
    pub fn advance(&mut self, 
        wait_for_inspector: Option<bool>,
        pump_v8_message_loop: Option<bool>,
    ) -> PyResult<bool> {
        let mut options= PollEventLoopOptions::default();
        if let Some(wait_for_inspector) = wait_for_inspector{
            options.wait_for_inspector = wait_for_inspector
        }
        if let Some(pump_v8_message_loop) = pump_v8_message_loop {
            options.pump_v8_message_loop = pump_v8_message_loop;
        }
        match self.runtime.get()?.advance_event_loop(options) {
            Ok(b) => {Ok(b)},
            Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
        }
    }

    pub fn eval(&mut self, code: &str) -> PyResult<Py<PyAny>>{
        let result: Result<serde_json::Value, _> = self.runtime.get()?.eval(code);
        match result {
            Ok(r) => {Ok(serde_to_python(r)?)}
            Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
        }
    }

    #[pyo3(signature=(name, *py_args))]
    pub fn call(&mut self, name:String, py_args: &Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
        let result:Result<serde_json::Value, _> = self.runtime.get()?.call_function_immediate(None, &name, &python_args_to_serde(py_args)?);
        match result {
            Ok(r) => {Ok(serde_to_python(r)?)}
            Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
        }
    }

    pub fn get_value(&mut self, name:String) -> PyResult<Py<PyAny>>{
        let result:Result<serde_json::Value, _> = self.runtime.get()?.get_value_immediate(None, &name);
        match result {
            Ok(r) => {Ok(serde_to_python(r)?)}
            Err(e) => {
                match e {
                    RSError::ValueNotFound(s) => {Err(PyKeyError::new_err(s))}
                    e =>
                        {Err(PyRuntimeError::new_err(e.to_string()))} 
                }
            }
        }
    }

    // #[pyo3(signature=(module, modules=None))]
    // pub fn load_modules(&mut self, module:Bound<'_, JsModule>, modules:Option<Vec<Py<JsModule>>>) -> PyResult<JsHandle> {
    //     let modules = modules.unwrap_or(Vec::new()).into_iter().map(|js| js.get()).collect();
    //     let res = self.runtime.get()?.load_modules(&module.get()?.module.get()?, modules);
    // }

    // TODO: In a future update implement asynchronous functions and js modules
    // along with making sure python can easily share values...
    // #[pyo3(signature=(name, *py_args))]
    // pub async fn call_async(&mut self, name:String, py_args: &Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
    //     let mut rt = self.runtime.get()?;
    //     let args = python_args_to_serde(py_args)?;
    //     let res = rt.call_function_async(None, &name, &args);

    //     match res.await {
    //         Ok(r) => {Ok(serde_to_python(r)?)}
    //         Err(e) => {Err(PyRuntimeError::new_err(e.to_string()))}
    //     }
    // }
}


#[pymodule]
pub fn pyrv8(module: &Bound<'_, PyModule>) -> PyResult<()>{
    module.add_class::<Context>()?;
    // module.add_class::<JsModule>()?;
    // module.add_class::<JsHandle>()?;
    Ok(())
}

// // /// A Python module implemented in Rust.
// #[pymodule]
// mod pyrv8 {
//     use pyo3::prelude::*;

//     /// Formats the sum of two numbers as string.
//     #[pyfunction]
//     fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//         Ok((a + b).to_string())
//     }
// }
