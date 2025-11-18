use extendr_api::prelude::*;
use wasmer::{Instance, Module, Store, Value, Function, imports, wat2wasm};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

mod memory;
mod host_functions;
mod type_converter;

use memory::WasmerMemoryManager;
use host_functions::WasmerHostFunctions;
use type_converter::TypeConverter;

/// A Wasmer WebAssembly runtime wrapper for R
pub struct WasmerRuntime {
    store: Store,
    modules: HashMap<String, Module>,
    instances: HashMap<String, Instance>,
    #[allow(dead_code)]
    memory_manager: WasmerMemoryManager,
}

impl WasmerRuntime {
    fn new() -> Self {
        Self {
            store: Store::default(),
            modules: HashMap::new(),
            instances: HashMap::new(),
            memory_manager: WasmerMemoryManager::new(),
        }
    }
}

// Global runtime instance using thread-safe static
static RUNTIME: OnceLock<Mutex<Option<WasmerRuntime>>> = OnceLock::new();

/// Initialize the Wasmer runtime
/// @export
#[extendr]
fn wasmer_init() -> String {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    *runtime_guard = Some(WasmerRuntime::new());
    "Wasmer runtime initialized".to_string()
}

/// Compile a WebAssembly module from WAT (WebAssembly Text) format
/// @param wat_code String containing WebAssembly text format code
/// @param module_name String name to identify this module
/// @export
#[extendr]
fn wasmer_compile_wat(wat_code: String, module_name: String) -> String {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        match wat2wasm(wat_code.as_bytes()) {
            Ok(wasm_bytes) => {
                match Module::new(&runtime.store, wasm_bytes) {
                    Ok(module) => {
                        runtime.modules.insert(module_name.clone(), module);
                        format!("Module '{}' compiled successfully", module_name)
                    }
                    Err(e) => format!("Error compiling module: {}", e),
                }
            }
            Err(e) => format!("Error converting WAT to WASM: {}", e),
        }
    } else {
        "Runtime not initialized. Call wasmer_init() first.".to_string()
    }
}

/// Instantiate a compiled WebAssembly module
/// @param module_name String name of the module to instantiate
/// @param instance_name String name to identify this instance
/// @export
#[extendr]
fn wasmer_instantiate(module_name: String, instance_name: String) -> String {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        if let Some(module) = runtime.modules.get(&module_name) {
            let import_object = imports! {};
            match Instance::new(&mut runtime.store, module, &import_object) {
                Ok(instance) => {
                    runtime.instances.insert(instance_name.clone(), instance);
                    format!("Instance '{}' created successfully", instance_name)
                }
                Err(e) => format!("Error creating instance: {}", e),
            }
        } else {
            format!("Module '{}' not found", module_name)
        }
    } else {
        "Runtime not initialized. Call wasmer_init() first.".to_string()
    }
}

/// Call an exported function from a WebAssembly instance
/// @param instance_name String name of the instance
/// @param function_name String name of the function to call
/// @param args List of arguments (integers and floats supported)
/// @export
#[extendr]
fn wasmer_call_function(instance_name: String, function_name: String, args: List) -> List {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        if let Some(instance) = runtime.instances.get(&instance_name) {
            if let Ok(func) = instance.exports.get_function(&function_name) {
                // Convert R arguments to Wasm values
                let mut wasm_args = Vec::new();
                for (_name, arg) in args.iter() {
                    match arg.rtype() {
                        Rtype::Integers => {
                            if let Some(val) = arg.as_integer() {
                                wasm_args.push(Value::I32(val));
                            }
                        }
                        Rtype::Doubles => {
                            if let Some(val) = arg.as_real() {
                                wasm_args.push(Value::F64(val));
                            }
                        }
                        _ => {}
                    }
                }

                match func.call(&mut runtime.store, &wasm_args) {
                    Ok(results) => {
                        let result_list = List::from_names_and_values(
                            ["success", "values"],
                            [r!(true), convert_wasm_values_to_r(results)],
                        ).unwrap();
                        result_list
                    }
                    Err(e) => {
                        List::from_names_and_values(
                            ["success", "error"],
                            [r!(false), r!(format!("Error calling function: {}", e))],
                        ).unwrap()
                    }
                }
            } else {
                List::from_names_and_values(
                    ["success", "error"],
                    [r!(false), r!(format!("Function '{}' not found", function_name))],
                ).unwrap()
            }
        } else {
            List::from_names_and_values(
                ["success", "error"],
                [r!(false), r!(format!("Instance '{}' not found", instance_name))],
            ).unwrap()
        }
    } else {
        List::from_names_and_values(
            ["success", "error"],
            [r!(false), r!("Runtime not initialized. Call wasmer_init() first.")],
        ).unwrap()
    }
}

/// Get list of exported functions from an instance
/// @param instance_name String name of the instance
/// @export
#[extendr]
fn wasmer_list_exports(instance_name: String) -> List {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let runtime_guard = runtime_mutex.lock().unwrap();
    
    if let Some(ref runtime) = runtime_guard.as_ref() {
        if let Some(instance) = runtime.instances.get(&instance_name) {
            let mut exports = Vec::new();
            for (name, _) in instance.exports.iter() {
                exports.push(name.clone());
            }
            List::from_names_and_values(
                ["success", "exports"],
                [r!(true), r!(exports)],
            ).unwrap()
        } else {
            List::from_names_and_values(
                ["success", "error"],
                [r!(false), r!(format!("Instance '{}' not found", instance_name))],
            ).unwrap()
        }
    } else {
        List::from_names_and_values(
            ["success", "error"],
            [r!(false), r!("Runtime not initialized. Call wasmer_init() first.")],
        ).unwrap()
    }
}

/// Create a simple "Hello World" example
/// @export
#[extendr]
fn wasmer_hello_world_example() -> String {
    let wat_code = r#"
(module
  (func $hello (export "hello") (result i32)
    i32.const 42)
)"#;
    
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if runtime_guard.is_none() {
        *runtime_guard = Some(WasmerRuntime::new());
    }
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        match wat2wasm(wat_code.as_bytes()) {
            Ok(wasm_bytes) => {
                match Module::new(&runtime.store, wasm_bytes) {
                    Ok(module) => {
                        let import_object = imports! {};
                        match Instance::new(&mut runtime.store, &module, &import_object) {
                            Ok(instance) => {
                                if let Ok(hello_func) = instance.exports.get_typed_function::<(), i32>(&runtime.store, "hello") {
                                    match hello_func.call(&mut runtime.store) {
                                        Ok(result) => format!("Hello World! Function returned: {}", result),
                                        Err(e) => format!("Error calling function: {}", e),
                                    }
                                } else {
                                    "Could not get hello function".to_string()
                                }
                            }
                            Err(e) => format!("Error creating instance: {}", e),
                        }
                    }
                    Err(e) => format!("Error compiling module: {}", e),
                }
            }
            Err(e) => format!("Error converting WAT: {}", e),
        }
    } else {
        "Could not initialize runtime".to_string()
    }
}

/// Math operations example
/// @param a First number
/// @param b Second number
/// @export
#[extendr]
fn wasmer_math_example(a: i32, b: i32) -> List {
    let wat_code = r#"
(module
  (func $add (export "add") (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add)
  (func $multiply (export "multiply") (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.mul)
)"#;

    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if runtime_guard.is_none() {
        *runtime_guard = Some(WasmerRuntime::new());
    }
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        match wat2wasm(wat_code.as_bytes()) {
            Ok(wasm_bytes) => {
                match Module::new(&runtime.store, wasm_bytes) {
                    Ok(module) => {
                        let import_object = imports! {};
                        match Instance::new(&mut runtime.store, &module, &import_object) {
                            Ok(instance) => {
                                let mut results = Vec::new();
                                
                                if let Ok(add_func) = instance.exports.get_typed_function::<(i32, i32), i32>(&runtime.store, "add") {
                                    if let Ok(result) = add_func.call(&mut runtime.store, a, b) {
                                        results.push(("add", result));
                                    }
                                }
                                
                                if let Ok(mul_func) = instance.exports.get_typed_function::<(i32, i32), i32>(&runtime.store, "multiply") {
                                    if let Ok(result) = mul_func.call(&mut runtime.store, a, b) {
                                        results.push(("multiply", result));
                                    }
                                }
                                
                                let names: Vec<&str> = results.iter().map(|(name, _)| *name).collect();
                                let values: Vec<i32> = results.iter().map(|(_, value)| *value).collect();
                                
                                List::from_names_and_values(names, values.iter().map(|&v| r!(v))).unwrap()
                            }
                            Err(e) => {
                                List::from_names_and_values(["error"], [r!(format!("Error creating instance: {}", e))]).unwrap()
                            }
                        }
                    }
                    Err(e) => {
                        List::from_names_and_values(["error"], [r!(format!("Error compiling module: {}", e))]).unwrap()
                    }
                }
            }
            Err(e) => {
                List::from_names_and_values(["error"], [r!(format!("Error converting WAT: {}", e))]).unwrap()
            }
        }
    } else {
        List::from_names_and_values(["error"], [r!("Could not initialize runtime")]).unwrap()
    }
}

/// Create an instance with host functions for mathematical operations
/// @param module_name String name of the module to instantiate
/// @param instance_name String name to identify this instance
/// @export
#[extendr]
fn wasmer_instantiate_with_math_imports(module_name: String, instance_name: String) -> String {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        if let Some(module) = runtime.modules.get(&module_name) {
            let math_functions = WasmerHostFunctions::create_math_functions(&mut runtime.store);
            
            let import_object = imports! {
                "env" => {
                    "square" => math_functions.get("square").unwrap().clone(),
                    "cube" => math_functions.get("cube").unwrap().clone(),
                    "factorial" => math_functions.get("factorial").unwrap().clone(),
                    "log" => WasmerHostFunctions::create_log_function(&mut runtime.store),
                    "timestamp" => WasmerHostFunctions::create_timestamp_function(&mut runtime.store),
                    "random" => WasmerHostFunctions::create_random_function(&mut runtime.store),
                }
            };
            
            match Instance::new(&mut runtime.store, module, &import_object) {
                Ok(instance) => {
                    runtime.instances.insert(instance_name.clone(), instance);
                    format!("Instance '{}' created with math imports", instance_name)
                }
                Err(e) => format!("Error creating instance: {}", e),
            }
        } else {
            format!("Module '{}' not found", module_name)
        }
    } else {
        "Runtime not initialized. Call wasmer_init() first.".to_string()
    }
}

/// Advanced function calling with type safety
/// @param instance_name String name of the instance
/// @param function_name String name of the function to call
/// @param args List of arguments with proper type conversion
/// @export
#[extendr]
fn wasmer_call_function_safe(instance_name: String, function_name: String, args: List) -> List {
    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        if let Some(instance) = runtime.instances.get(&instance_name) {
            if let Ok(func) = instance.exports.get_function(&function_name) {
                // Convert R arguments to Wasm values using the type converter
                match TypeConverter::r_vector_to_wasm(args) {
                    Ok(wasm_args) => {
                        match func.call(&mut runtime.store, &wasm_args) {
                            Ok(results) => {
                                List::from_names_and_values(
                                    ["success", "values"],
                                    [r!(true), TypeConverter::wasm_vector_to_r(&results)],
                                ).unwrap()
                            }
                            Err(e) => {
                                List::from_names_and_values(
                                    ["success", "error"],
                                    [r!(false), r!(format!("Error calling function: {}", e))],
                                ).unwrap()
                            }
                        }
                    }
                    Err(e) => {
                        List::from_names_and_values(
                            ["success", "error"],
                            [r!(false), r!(format!("Type conversion error: {}", e))],
                        ).unwrap()
                    }
                }
            } else {
                List::from_names_and_values(
                    ["success", "error"],
                    [r!(false), r!(format!("Function '{}' not found", function_name))],
                ).unwrap()
            }
        } else {
            List::from_names_and_values(
                ["success", "error"],
                [r!(false), r!(format!("Instance '{}' not found", instance_name))],
            ).unwrap()
        }
    } else {
        List::from_names_and_values(
            ["success", "error"],
            [r!(false), r!("Runtime not initialized. Call wasmer_init() first.")],
        ).unwrap()
    }
}

/// Example with host function imports
/// @export
#[extendr]
fn wasmer_host_function_example() -> List {
    let wat_code = r#"
(module
  (func $square (import "env" "square") (param i32) (result i32))
  (func $log (import "env" "log") (param i32))
  (func $timestamp (import "env" "timestamp") (result i64))
  
  (func $test_host_functions (export "test") (param $x i32) (result i32)
    ;; Log the input
    (call $log (local.get $x))
    
    ;; Square the input and return
    (call $square (local.get $x))
  )
  
  (func $get_time (export "get_time") (result i64)
    (call $timestamp)
  )
)"#;

    let runtime_mutex = RUNTIME.get_or_init(|| Mutex::new(None));
    let mut runtime_guard = runtime_mutex.lock().unwrap();
    
    if runtime_guard.is_none() {
        *runtime_guard = Some(WasmerRuntime::new());
    }
    
    if let Some(ref mut runtime) = runtime_guard.as_mut() {
        match wat2wasm(wat_code.as_bytes()) {
            Ok(wasm_bytes) => {
                match Module::new(&runtime.store, wasm_bytes) {
                    Ok(module) => {
                        let import_object = imports! {
                            "env" => {
                                "square" => Function::new_typed(&mut runtime.store, |x: i32| -> i32 { x * x }),
                                "log" => Function::new_typed(&mut runtime.store, |x: i32| {
                                    rprintln!("WASM logged: {}", x);
                                }),
                                "timestamp" => Function::new_typed(&mut runtime.store, || -> i64 {
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs() as i64
                                }),
                            }
                        };
                        
                        match Instance::new(&mut runtime.store, &module, &import_object) {
                            Ok(instance) => {
                                let mut results = std::collections::HashMap::new();
                                
                                // Test the function
                                if let Ok(test_func) = instance.exports.get_typed_function::<i32, i32>(&runtime.store, "test") {
                                    if let Ok(result) = test_func.call(&mut runtime.store, 5) {
                                        results.insert("test_result".to_string(), r!(result));
                                    }
                                }
                                
                                // Get timestamp
                                if let Ok(time_func) = instance.exports.get_typed_function::<(), i64>(&runtime.store, "get_time") {
                                    if let Ok(timestamp) = time_func.call(&mut runtime.store) {
                                        results.insert("timestamp".to_string(), r!(timestamp as f64));
                                    }
                                }
                                
                                List::from_names_and_values(
                                    ["success", "results"],
                                    [r!(true), {
                                        let names: Vec<String> = results.keys().cloned().collect();
                                        let values: Vec<Robj> = results.values().cloned().collect();
                                        List::from_names_and_values(names.iter().map(|s| s.as_str()).collect::<Vec<_>>(), values).unwrap().into()
                                    }],
                                ).unwrap()
                            }
                            Err(e) => {
                                List::from_names_and_values(
                                    ["success", "error"],
                                    [r!(false), r!(format!("Error creating instance: {}", e))],
                                ).unwrap()
                            }
                        }
                    }
                    Err(e) => {
                        List::from_names_and_values(
                            ["success", "error"],
                            [r!(false), r!(format!("Error compiling module: {}", e))],
                        ).unwrap()
                    }
                }
            }
            Err(e) => {
                List::from_names_and_values(
                    ["success", "error"],
                    [r!(false), r!(format!("Error converting WAT: {}", e))],
                ).unwrap()
            }
        }
    } else {
        List::from_names_and_values(
            ["success", "error"],
            [r!(false), r!("Could not initialize runtime")],
        ).unwrap()
    }
}

// Helper function to convert Wasm values to R values
fn convert_wasm_values_to_r(values: Box<[Value]>) -> Robj {
    let mut r_values = Vec::new();
    for value in values.iter() {
        match value {
            Value::I32(i) => r_values.push(r!(*i)),
            Value::I64(i) => r_values.push(r!(*i as i32)), // Convert to i32 for R
            Value::F32(f) => r_values.push(r!(*f as f64)), // Convert to f64 for R
            Value::F64(f) => r_values.push(r!(*f)),
            _ => r_values.push(r!(0)), // Default for unsupported types
        }
    }
    if r_values.len() == 1 {
        r_values.into_iter().next().unwrap()
    } else {
        r!(r_values)
    }
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
extendr_module! {
    mod wasmer;
    fn wasmer_init;
    fn wasmer_compile_wat;
    fn wasmer_instantiate;
    fn wasmer_instantiate_with_math_imports;
    fn wasmer_call_function;
    fn wasmer_call_function_safe;
    fn wasmer_list_exports;
    fn wasmer_hello_world_example;
    fn wasmer_math_example;
    fn wasmer_host_function_example;
}
