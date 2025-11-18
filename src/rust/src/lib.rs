use extendr_api::prelude::*;
use extendr_api::wrapper::ExternalPtr;
use wasmer::{Instance, Module, Store, Value, Function, imports, wat2wasm};
use std::collections::HashMap;

mod memory;
mod host_functions;
mod type_converter;
mod api;

use memory::WasmerMemoryManager;
use host_functions::WasmerHostFunctions;
use type_converter::TypeConverter;

/// Helper function to convert Wasm values to R values
fn convert_wasm_values_to_r(values: Box<[Value]>) -> Robj {
    let mut r_values = Vec::new();
    for value in values.iter() {
        match value {
            Value::I32(i) => r_values.push(r!(*i)),
            Value::I64(i) => r_values.push(r!(*i as i32)),
            Value::F32(f) => r_values.push(r!(*f as f64)),
            Value::F64(f) => r_values.push(r!(*f)),
            _ => r_values.push(r!(0)),
        }
    }
    if r_values.len() == 1 {
        r_values.into_iter().next().unwrap()
    } else {
        r!(r_values)
    }
}

/// A Wasmer WebAssembly runtime wrapper for R
pub struct WasmerRuntime {
    store: Store,
    modules: HashMap<String, Module>,
    instances: HashMap<String, Instance>,
    r_function_registry: HashMap<String, Robj>,
    #[allow(dead_code)]
    memory_manager: WasmerMemoryManager,
}

impl WasmerRuntime {
    fn new() -> Self {
        Self {
            store: Store::default(),
            modules: HashMap::new(),
            instances: HashMap::new(),
            r_function_registry: HashMap::new(),
            memory_manager: WasmerMemoryManager::new(),
        }
    }
}

// All other functions below are NOT marked #[extendr]
// They must be called from Rust or wrapped in R using external pointer logic
fn wasmer_compile_wat(runtime: &mut WasmerRuntime, wat_code: String, module_name: String) -> String {
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
}

fn wasmer_instantiate(runtime: &mut WasmerRuntime, module_name: String, instance_name: String) -> String {
    if let Some(module) = runtime.modules.get(&module_name) {
        let import_object = imports! {
            "env" => {
                "r_host_call" => api::create_generic_r_host_function(&mut runtime.store),
            }
        };
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
}

fn wasmer_call_function(runtime: &mut WasmerRuntime, instance_name: String, function_name: String, args: List) -> List {
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(func) = instance.exports.get_function(&function_name) {
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
}

fn wasmer_list_exports(runtime: &mut WasmerRuntime, instance_name: String) -> List {
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
}

/// Create a simple "Hello World" example
/// @param runtime External pointer to WasmerRuntime
/// @return String result from WASM hello function
/// @export
#[extendr]
pub fn wasmer_hello_world_example_ext(mut ptr: ExternalPtr<WasmerRuntime>) -> String {
    let runtime = ptr.as_mut();
    let wat_code = r#"
(module
  (func $hello (export "hello") (result i32)
    i32.const 42)
)"#;
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
}

/// Math operations example
/// @param runtime External pointer to WasmerRuntime
/// @param a First integer
/// @param b Second integer
/// @return List with results of add and multiply
/// @export
#[extendr]
pub fn wasmer_math_example_ext(mut ptr: ExternalPtr<WasmerRuntime>, a: i32, b: i32) -> List {
    let runtime = ptr.as_mut();
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
}

/// Create an instance with host functions for mathematical operations
/// @param runtime External pointer to WasmerRuntime
/// @param module_name String name of the module to instantiate
/// @param instance_name String name to identify this instance
/// @return Status message
/// @export
#[extendr]
pub fn wasmer_instantiate_with_math_imports_ext(mut ptr: ExternalPtr<WasmerRuntime>, module_name: String, instance_name: String) -> String {
    let runtime = ptr.as_mut();
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
}

/// Advanced function calling with type safety
/// @param runtime External pointer to WasmerRuntime
/// @param instance_name String name of the instance
/// @param function_name String name of the function to call
/// @param args List of arguments with proper type conversion
/// @return List with success flag and result or error
/// @export
#[extendr]
pub fn wasmer_call_function_safe_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, function_name: String, args: List) -> List {
    let runtime = ptr.as_mut();
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
}

/// Example with host function imports
/// @param runtime External pointer to WasmerRuntime
/// @return List with results
/// @export
#[extendr]
pub fn wasmer_host_function_example_ext(mut ptr: ExternalPtr<WasmerRuntime>) -> List {
    let runtime = ptr.as_mut();
    let wat_code = r#"
(module
  (func $square (import "env" "square") (param i32) (result i32))
  (func $log (import "env" "log") (param i32))
  (func $timestamp (import "env" "timestamp") (result i64))
  (func $test_host_functions (export "test") (param $x i32) (result i32)
    (call $log (local.get $x))
    (call $square (local.get $x))
  )
  (func $get_time (export "get_time") (result i64)
    (call $timestamp)
  )
)"#;
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
                            if let Ok(test_func) = instance.exports.get_typed_function::<i32, i32>(&runtime.store, "test") {
                                if let Ok(result) = test_func.call(&mut runtime.store, 5) {
                                    results.insert("test_result".to_string(), r!(result));
                                }
                            }
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
                    List::from_names_and_values(["success", "error"], [r!(false), r!(format!("Error compiling module: {}", e))]).unwrap()
                }
            }
        }
        Err(e) => {
            List::from_names_and_values(["success", "error"], [r!(false), r!(format!("Error converting WAT: {}", e))]).unwrap()
        }
    }
}

/// Create a new Wasmer runtime for R. Returns an external pointer to the runtime object.
/// @return External pointer to WasmerRuntime
/// @export
#[extendr]
pub fn wasmer_runtime_new() -> ExternalPtr<WasmerRuntime> {
    ExternalPtr::new(WasmerRuntime::new())
}

/// Compile a WAT (WebAssembly Text) module and add it to the runtime.
/// @param ptr External pointer to WasmerRuntime
/// @param wat_code WAT code as a string
/// @param module_name Name to register the module under
/// @return Status message
/// @export
#[extendr]
pub fn wasmer_compile_wat_ext(mut ptr: ExternalPtr<WasmerRuntime>, wat_code: String, module_name: String) -> String {
    let runtime = ptr.as_mut();
    wasmer_compile_wat(runtime, wat_code, module_name)
}

/// Compile a WASM binary and add it to the runtime.
/// @param ptr External pointer to WasmerRuntime
/// @param wasm_bytes WASM binary as R raw vector
/// @param module_name Name to register the module under
/// @return Status message
/// @export
#[extendr]
pub fn wasmer_compile_wasm_ext(mut ptr: ExternalPtr<WasmerRuntime>, wasm_bytes: Robj, module_name: String) -> String {
    let runtime = ptr.as_mut();
    let bytes: Vec<u8> = match wasm_bytes.as_raw() {
        Some(slice) => slice.as_slice().to_vec(),
        None => return "Input is not a raw vector".to_string(),
    };
    match Module::new(&runtime.store, &bytes) {
        Ok(module) => {
            runtime.modules.insert(module_name.clone(), module);
            format!("Module '{}' compiled from binary successfully", module_name)
        }
        Err(e) => format!("Error compiling module from binary: {}", e),
    }
}

/// Instantiate a compiled module in the runtime.
/// @param ptr External pointer to WasmerRuntime
/// @param module_name Name of the module to instantiate
/// @param instance_name Name to register the instance under
/// @return Status message
/// @export
#[extendr]
pub fn wasmer_instantiate_ext(mut ptr: ExternalPtr<WasmerRuntime>, module_name: String, instance_name: String) -> String {
    let runtime = ptr.as_mut();
    wasmer_instantiate(runtime, module_name, instance_name)
}

/// Call an exported function from a WASM instance.
/// @param ptr External pointer to WasmerRuntime
/// @param instance_name Name of the instance
/// @param function_name Name of the function to call
/// @param args Arguments as R list
/// @return List with success flag and result or error
/// @export
#[extendr]
pub fn wasmer_call_function_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, function_name: String, args: List) -> List {
    let runtime = ptr.as_mut();
    wasmer_call_function(runtime, instance_name, function_name, args)
}

/// List all exports from a WASM instance.
/// @param ptr External pointer to WasmerRuntime
/// @param instance_name Name of the instance
/// @return List with success flag and exports or error
/// @export
#[extendr]
pub fn wasmer_list_exports_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String) -> List {
    let runtime = ptr.as_mut();
    wasmer_list_exports(runtime, instance_name)
}

/// Register an R function for use as a host function in WASM (per-runtime)
/// @param ptr External pointer to WasmerRuntime
/// @param name Name to register the function under
/// @param fun R function object
/// @return TRUE if successful
/// @export
#[extendr]
pub fn wasmer_register_r_function_ext(mut ptr: ExternalPtr<WasmerRuntime>, name: String, fun: Robj) -> bool {
    let runtime = ptr.as_mut();
    runtime.r_function_registry.insert(name, fun);
    true
}

/// Convert WAT (WebAssembly Text) to WASM binary and return as R raw vector
/// @param wat_code WAT code as a string
/// @return WASM binary as R raw vector, or error string if conversion fails
/// @export
#[extendr]
pub fn wasmer_wat_to_wasm_ext(wat_code: String) -> Robj {
    match wasmer::wat2wasm(wat_code.as_bytes()) {
        Ok(wasm_bytes) => r!(wasm_bytes.into_owned()),
        Err(e) => r!(format!("Error converting WAT to WASM: {}", e)),
    }
}



extendr_module! {
    mod wasmer;
    fn wasmer_runtime_new;
    fn wasmer_compile_wat_ext;
    fn wasmer_compile_wasm_ext;
    fn wasmer_instantiate_ext;
    fn wasmer_call_function_ext;
    fn wasmer_list_exports_ext;
    fn wasmer_register_r_function_ext;
    fn wasmer_math_example_ext;
    fn wasmer_hello_world_example_ext;
    fn wasmer_wat_to_wasm_ext;
    fn wasmer_instantiate_with_math_imports_ext;
    fn wasmer_call_function_safe_ext;
    fn wasmer_host_function_example_ext;
}
