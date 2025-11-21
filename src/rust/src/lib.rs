use wasmer::{Function, Store, Instance, FunctionEnv, FunctionEnvMut, Module, Value, imports, wat2wasm};
use wasmer::{Table, TableType, Type};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use extendr_api::prelude::*;
use extendr_api::wrapper::ExternalPtr;
use std::collections::HashMap;
use wasmer::{AsStoreRef, AsStoreMut};
use wasmer::sys::EngineBuilder;
use std::sync::atomic::{AtomicU32, Ordering};
use wasmer_wasix::WasiFunctionEnv;
use memory::WasmerMemoryManager;
use host_functions::WasmerHostFunctions;
use type_converter::TypeConverter;
use wasi_utils::WasiUtils;
use compiler_utils::CompilerUtils;
mod memory;
mod host_functions;
mod type_converter;
mod wasi_utils;
mod compiler_utils;


thread_local! {
    static R_FUNCTION_REGISTRY: Lazy<RefCell<HashMap<u32, Robj>>> =
        Lazy::new(|| RefCell::new(HashMap::new()));
}

static TOKIO_RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().unwrap()
});

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn register_r_function_internal(fun: Robj) -> u32 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    R_FUNCTION_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(id, fun);
    });
    id
}

pub fn register_r_function(_name: &str, fun: Robj) -> u32 {
    let id = register_r_function_internal(fun);
    id
}


fn read_i32_args_from_memory(instance: &Instance, store: &wasmer::StoreRef, ptr: i32, argc: i32) -> Vec<i32> {
    let memory = instance.exports.get_memory("memory").unwrap();
    let view = memory.view(store);
    (0..argc)
        .map(|i| {
            let offset = ptr + i * 4;
            let bytes = [
                unsafe { *view.data_ptr().add(offset as usize) },
                unsafe { *view.data_ptr().add((offset+1) as usize) },
                unsafe { *view.data_ptr().add((offset+2) as usize) },
                unsafe { *view.data_ptr().add((offset+3) as usize) },
            ];
            i32::from_le_bytes(bytes)
        })
        .collect()
}

#[derive(Clone)]
pub struct WasmerEnv {
    pub instance: Option<Instance>,
}



pub fn create_generic_r_host_function(env: &FunctionEnv<WasmerEnv>, store: &mut Store) -> Function {
    Function::new_typed_with_env(store, env, |mut env: FunctionEnvMut<WasmerEnv>, handle: i32, args_ptr: i32, argc: i32| -> i32 {
        let _store_mut = env.as_store_mut();
        let (env_data, store_mut) = env.data_and_store_mut();
        let instance = match env_data.instance.as_ref() {
            Some(i) => i,
            None => return 0,
        };
        let store_ref = store_mut.as_store_ref();
        
        // Read arguments
        let args = read_i32_args_from_memory(instance, &store_ref, args_ptr, argc);
        // Diagnostics
        rprintln!("[wasmer] Host call: handle={}, args={:?}", handle, args);
        
        // Lookup and call R function
        let result = R_FUNCTION_REGISTRY.with(|reg| {
            reg.borrow().get(&(handle as u32)).cloned()
        }).and_then(|rfun| {
            rprintln!("[wasmer] Found R function for handle {}, calling...", handle);
            if args.len() == 1 {
                rfun.call(pairlist!(args[0])).ok()
            } else {
                let r_args = args.clone().into_iter().map(|x| r!(x)).collect::<Vec<Robj>>();
                rfun.call(pairlist!(r_args)).ok()
            }
        });
        if let Some(r) = result {
            rprintln!("[wasmer] R call result: {:?}", r);
            if let Some(val) = r.as_integer() {
                rprintln!("[wasmer] Returning integer value: {}", val);
                return val;
            } else {
                rprintln!("[wasmer] R function for handle {} did not return an integer: {:?}", handle, r);
            }
        } else {
            rprintln!("[wasmer] R function for handle {} not found or call failed.", handle);
        }
        0 // fallback
    })
}



/// Helper function to convert Wasm values to R values
fn convert_wasm_values_to_r(values: Box<[Value]>) -> Robj {
    let mut r_values = Vec::new();
    for value in values.iter() {
        match value {
            Value::I32(i) => r_values.push(r!(*i)),
            Value::I64(i) => r_values.push(r!(*i as f64)),
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
    env: Option<FunctionEnv<WasmerEnv>>,
    #[allow(dead_code)]
    memory_manager: WasmerMemoryManager,
    wasi_env: Option<WasiFunctionEnv>,
    shutdown: bool,
}

impl WasmerRuntime {
    fn new() -> Self {
        Self {
            store: Store::default(),
            modules: HashMap::new(),
            instances: HashMap::new(),
            r_function_registry: HashMap::new(),
            env: None,
            memory_manager: WasmerMemoryManager::new(),
            wasi_env: None,
            shutdown: false,
        }
    }

    /// Check if the runtime has been shutdown
    fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Mark the runtime as shutdown
    fn set_shutdown(&mut self) {
        self.shutdown = true;
    }

    /// Explicitly shutdown the runtime, freeing all modules, instances, registries, and store
    /// This function is idempotent and safe to call multiple times.
    pub fn shutdown_and_finalize(ptr: &mut ExternalPtr<Self>) {
        let runtime = ptr.as_mut();
        if !runtime.is_shutdown() {
            runtime.modules.clear();
            runtime.instances.clear();
            runtime.r_function_registry.clear();
            runtime.env = None;
            runtime.wasi_env = None;
            runtime.set_shutdown();
        }
        // No need to manually set the external pointer address to NULL.
        // The finalizer will handle cleanup when the R object is collected.
    }

    /// Explicitly shutdown the runtime, freeing all modules, instances, and registries
    pub fn shutdown(&mut self) {
        self.modules.clear();
        self.instances.clear();
        self.r_function_registry.clear();
        self.env = None;
        self.wasi_env = None;
        // Optionally drop memory manager resources if needed
    }
}

// Release the resources held by the runtime before the gc collects the external pointer
/// Release resources held by the Wasmer runtime
///
/// @title Release Wasmer runtime resources
/// @description Explicitly shutdown the runtime, free resources, and clear the R external pointer.
/// @family runtime management
/// @seealso [wasmer_runtime_new()], [wasmer_runtime_new_with_compiler_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @return NULL (invisible)
/// @examples
/// wasmer_runtime_release_ressources(ptr)
/// @export
#[extendr]
pub fn wasmer_runtime_release_ressources(mut ptr: ExternalPtr<WasmerRuntime>) {
    WasmerRuntime::shutdown_and_finalize(&mut ptr);
}
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
        let env = FunctionEnv::new(&mut runtime.store, WasmerEnv { instance: None });
        let mut import_object = imports! {
            "env" => {
                "r_host_call" => create_generic_r_host_function(&env, &mut runtime.store),
            }
        };
        
        // Add WASI imports if enabled
        if let Some(wasi_env) = &runtime.wasi_env {
            let wasi_imports = wasi_env.import_object(&mut runtime.store, module).unwrap_or_else(|_| imports! {});
            // Merge imports - this is a bit tricky with the imports! macro structure
            // For now, we'll just extend the import object if possible or create a new one
            // Wasmer's ImportObject can be extended
            import_object.extend(&wasi_imports);
        }

        match Instance::new(&mut runtime.store, module, &import_object) {
            Ok(final_instance) => {
                env.as_mut(&mut runtime.store).instance = Some(final_instance.clone());
                runtime.instances.insert(instance_name.clone(), final_instance.clone());
                runtime.env = Some(env);
                
                // Initialize WASI if present
                if let Some(wasi_env) = &runtime.wasi_env {
                     // wasmer-wasix handles initialization automatically on first call usually, 
                     // but we might need to call initialize explicitly if we want to be sure.
                     // For now, let's assume it works as is.
                     match wasi_env.clone().initialize(&mut runtime.store, final_instance.clone()) {
                        Ok(_) => {},
                        Err(e) => return format!("Error initializing WASI: {}", e),
                     }
                }
                
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
                        [r!(true), convert_wasm_values_to_r(Box::<[Value]>::from(results))],
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

/// List exported function signatures (name, input types, output types) for a WASM instance
///
/// @title List WASM function signatures
/// @description List exported function signatures (name, input types, output types) for a WASM instance.
/// @family exports and signatures
/// @seealso [wasmer_list_exports_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @return Data frame with columns: name, params, results
/// @examples
/// wasmer_list_function_signatures_ext(ptr, "inst1")
/// @export
#[extendr]
pub fn wasmer_list_function_signatures_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String) -> List {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        let module = instance.module();
        let mut names = Vec::new();
        let mut params = Vec::new();
        let mut results = Vec::new();
        for export in module.exports() {
            if let wasmer::ExternType::Function(func_ty) = export.ty() {
                names.push(export.name().to_string());
                params.push(format!("{:?}", func_ty.params()));
                results.push(format!("{:?}", func_ty.results()));
            }
        }
        List::from_names_and_values(
            ["name", "params", "results"],
            [r!(names), r!(params), r!(results)]
        ).unwrap()
    } else {
        List::from_names_and_values(
            ["error"],
            [r!(format!("Instance '{}' not found", instance_name))]
        ).unwrap()
    }
}

/// Create a simple "Hello World" example
///
/// @title Hello World example
/// @description Create a simple WASM "Hello World" example.
/// @family function calling
/// @seealso [wasmer_call_function_ext()], [wasmer_call_function_safe_ext()], [wasmer_host_function_example_ext()], [wasmer_math_example_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @return String result from WASM hello function
/// @examples
/// wasmer_hello_world_example_ext(ptr)
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
///
/// @title Math operations example
/// @description Example WASM module for math operations.
/// @family function calling
/// @seealso [wasmer_call_function_ext()], [wasmer_call_function_safe_ext()], [wasmer_host_function_example_ext()], [wasmer_hello_world_example_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param a First integer.
/// @param b Second integer.
/// @return List with results of add and multiply
/// @examples
/// wasmer_math_example_ext(ptr, 2, 3)
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
///
/// @title Instantiate WASM module with math imports
/// @description Instantiate a WASM module with host functions for mathematical operations.
/// @family module instantiation
/// @seealso [wasmer_instantiate_ext()], [wasmer_instantiate_with_table_ext()], [wasmer_wasi_state_new_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param module_name String name of the module to instantiate.
/// @param instance_name String name to identify this instance.
/// @return Status message
/// @examples
/// wasmer_instantiate_with_math_imports_ext(ptr, "mod1", "inst1")
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
///
/// @title Call WASM function (type safe)
/// @description Call an exported WASM function with type safety and conversion.
/// @family function calling
/// @seealso [wasmer_call_function_ext()], [wasmer_host_function_example_ext()], [wasmer_math_example_ext()], [wasmer_hello_world_example_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name String name of the instance.
/// @param function_name String name of the function to call.
/// @param args List of arguments with proper type conversion.
/// @return List with success flag and result or error
/// @examples
/// wasmer_call_function_safe_ext(ptr, "inst1", "add", list(1, 2))
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
///
/// @title Host function example
/// @description Example with host function imports.
/// @family function calling
/// @seealso [wasmer_call_function_ext()], [wasmer_call_function_safe_ext()], [wasmer_math_example_ext()], [wasmer_hello_world_example_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @return List with results
/// @examples
/// wasmer_host_function_example_ext(ptr)
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

/// Create a new Wasmer runtime
///
/// @title Create a new Wasmer runtime
/// @description Create a new Wasmer runtime for executing WebAssembly modules.
/// @family runtime management
/// @seealso [wasmer_runtime_new_with_compiler_ext()], [wasmer_runtime_release_ressources()]
/// @return External pointer to WasmerRuntime
/// @examples
/// ptr <- wasmer_runtime_new()
/// @export
#[extendr]
pub fn wasmer_runtime_new() -> ExternalPtr<WasmerRuntime> {
    ExternalPtr::new(WasmerRuntime::new())
}

/// Create a new Wasmer runtime with a specific compiler
///
/// @title Create a new Wasmer runtime with a specific compiler
/// @description Create a new Wasmer runtime for executing WebAssembly modules using a specified compiler backend.
/// @family runtime management
/// @seealso [wasmer_runtime_new()], [wasmer_runtime_release_ressources()]
/// @param compiler_name Name of the compiler ("cranelift", "singlepass").
/// @return External pointer to WasmerRuntime
/// @examples
/// ptr <- wasmer_runtime_new_with_compiler_ext("cranelift")
/// @export
#[extendr]
pub fn wasmer_runtime_new_with_compiler_ext(compiler_name: String) -> ExternalPtr<WasmerRuntime> {
    let compiler_config = match CompilerUtils::get_compiler_config(&compiler_name) {
        Ok(c) => c,
        Err(e) => {
            rprintln!("Error getting compiler config: {}", e);
            return ExternalPtr::new(WasmerRuntime::new()); // Fallback to default
        }
    };
    // Note: WasmerRuntime::new() uses Store::default(). We need a way to pass the compiler.
    // We'll modify WasmerRuntime::new to accept an optional compiler config or create a new constructor.
    // Since WasmerRuntime struct definition is simple, we can just create it here.
    let engine = EngineBuilder::new(compiler_config).engine();
    let store = Store::new(engine);
    let runtime = WasmerRuntime {
        store,
        modules: HashMap::new(),
        instances: HashMap::new(),
        r_function_registry: HashMap::new(),
        env: None,
        memory_manager: WasmerMemoryManager::new(),
        wasi_env: None,
        shutdown: false,
    };
    ExternalPtr::new(runtime)
}

/// Create a WASI or WASIX state for the runtime
///
/// @title Create WASI/WASIX state
/// @description Create a WASI or WASIX state for the runtime.
/// @family module instantiation
/// @seealso [wasmer_instantiate_ext()], [wasmer_instantiate_with_math_imports_ext()], [wasmer_instantiate_with_table_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param module_name Name of the module (for WASI/WASIX args).
/// @param env_type Environment type: "wasi" (default) or "wasix".
/// @return TRUE if successful, FALSE otherwise
/// @examples
/// wasmer_wasi_state_new_ext(ptr, "mod1", "wasi")
/// @export
#[extendr]
pub fn wasmer_wasi_state_new_ext(mut ptr: ExternalPtr<WasmerRuntime>, module_name: String, env_type: Option<String>) -> bool {
    let runtime = ptr.as_mut();
    let _guard = TOKIO_RUNTIME.enter();
    let env_type = env_type.unwrap_or_else(|| "wasi".to_string());
    match env_type.as_str() {
        "wasix" => {
            match WasiUtils::create_wasi_env(&mut runtime.store, &module_name) {
                Ok(env) => {
                    runtime.wasi_env = Some(env);
                    true
                }
                Err(e) => {
                    rprintln!("Error creating WASIX state: {}", e);
                    false
                }
            }
        }
        _ => {
            match WasiUtils::create_wasi_env(&mut runtime.store, &module_name) {
                Ok(env) => {
                    runtime.wasi_env = Some(env);
                    true
                }
                Err(e) => {
                    rprintln!("Error creating WASI state: {}", e);
                    false
                }
            }
        }
    }
}


/// Compile a WAT (WebAssembly Text) module and add it to the runtime
///
/// @title Compile WAT module
/// @description Compile a WebAssembly Text (WAT) module and add it to the runtime.
/// @family module compilation
/// @seealso [wasmer_compile_wasm_ext()], [wasmer_wat_to_wasm_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param wat_code WAT code as a string.
/// @param module_name Name to register the module under.
/// @return Status message
/// @examples
/// wasmer_compile_wat_ext(ptr, wat_code, "mod1")
/// @export
#[extendr]
pub fn wasmer_compile_wat_ext(mut ptr: ExternalPtr<WasmerRuntime>, wat_code: String, module_name: String) -> String {
    let runtime = ptr.as_mut();
    wasmer_compile_wat(runtime, wat_code, module_name)
}

/// Compile a WASM binary and add it to the runtime
///
/// @title Compile WASM binary
/// @description Compile a WebAssembly binary and add it to the runtime.
/// @family module compilation
/// @seealso [wasmer_compile_wat_ext()], [wasmer_wat_to_wasm_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param wasm_bytes WASM binary as R raw vector.
/// @param module_name Name to register the module under.
/// @return Status message
/// @examples
/// wasmer_compile_wasm_ext(ptr, wasm_bytes, "mod1")
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

/// Call an exported function from a WASM instance
///
/// @title Call WASM function
/// @description Call an exported function from a WASM instance.
/// @family function calling
/// @seealso [wasmer_call_function_safe_ext()], [wasmer_host_function_example_ext()], [wasmer_math_example_ext()], [wasmer_hello_world_example_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param function_name Name of the function to call.
/// @param args Arguments as R list.
/// @return List with success flag and result or error
/// @examples
/// wasmer_call_function_ext(ptr, "inst1", "add", list(1, 2))
/// @export
#[extendr]
pub fn wasmer_call_function_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, function_name: String, args: List) -> List {
    let runtime = ptr.as_mut();
    wasmer_call_function(runtime, instance_name, function_name, args)
}

/// List all exports from a WASM instance
///
/// @title List WASM exports
/// @description List all exports from a WASM instance.
/// @family exports and signatures
/// @seealso [wasmer_list_function_signatures_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @return List with success flag and exports or error
/// @examples
/// wasmer_list_exports_ext(ptr, "inst1")
/// @export
#[extendr]
pub fn wasmer_list_exports_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String) -> List {
    let runtime = ptr.as_mut();
    wasmer_list_exports(runtime, instance_name)
}

/// Register an R function for use as a host function in WASM (per-runtime)
///
/// @title Register R host function
/// @description Register an R function for use as a host function in WASM (per-runtime).
/// @family host function registration
/// @seealso [wasmer_function_new_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_i32_to_void()], [wasmer_function_new_void_to_i32()]
/// @param ptr External pointer to WasmerRuntime.
/// @param _name Name to register the function under.
/// @param fun R function object.
/// @return TRUE if successful
/// @examples
/// wasmer_register_r_function_ext(ptr, "myfun", function(x) x)
/// @export
#[extendr]
pub fn wasmer_register_r_function_ext(mut ptr: ExternalPtr<WasmerRuntime>, name: String, fun: Robj) -> u32 {
    let runtime = ptr.as_mut();
    runtime.r_function_registry.insert(name.clone(), fun.clone());
    let handle = register_r_function(&name, fun); // Ensure global registry is updated
    handle
}

/// Convert WAT (WebAssembly Text) to WASM binary and return as R raw vector
///
/// @title Convert WAT to WASM
/// @description Convert WebAssembly Text (WAT) to WASM binary and return as R raw vector.
/// @family module compilation
/// @seealso [wasmer_compile_wat_ext()], [wasmer_compile_wasm_ext()]
/// @param wat_code WAT code as a string.
/// @return WASM binary as R raw vector, or error string if conversion fails
/// @examples
/// wasmer_wat_to_wasm_ext(wat_code)
/// @export
#[extendr]
pub fn wasmer_wat_to_wasm_ext(wat_code: String) -> Robj {
    match wasmer::wat2wasm(wat_code.as_bytes()) {
        Ok(wasm_bytes) => r!(wasm_bytes.into_owned()),
        Err(e) => r!(format!("Error converting WAT to WASM: {}", e)),
    }
}

/// Get the size of exported memory (in bytes and pages)
///
/// @title Get WASM memory size
/// @description Get the size of exported memory (in bytes and pages).
/// @family memory operations
/// @seealso [wasmer_memory_read_ext()], [wasmer_memory_write_ext()], [wasmer_memory_read_string_ext()], [wasmer_memory_grow_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param memory_name Name of the exported memory (default "memory").
/// @return List with size_bytes and size_pages
/// @examples
/// wasmer_memory_size_ext(ptr, "inst1", "memory")
/// @export
#[extendr]
pub fn wasmer_memory_size_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, memory_name: String) -> List {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(memory) = instance.exports.get_memory(&memory_name) {
            let view = memory.view(&runtime.store);
            List::from_names_and_values(
                ["size_bytes", "size_pages"],
                [r!(view.data_size()), r!(view.size().0)]
            ).unwrap()
        } else {
            List::from_names_and_values(["error"], [r!("Memory not found")]).unwrap()
        }
    } else {
        List::from_names_and_values(["error"], [r!("Instance not found")]).unwrap()
    }
}

/// Read bytes from WASM memory
///
/// @title Read WASM memory
/// @description Read bytes from WASM memory.
/// @family memory operations
/// @seealso [wasmer_memory_size_ext()], [wasmer_memory_write_ext()], [wasmer_memory_read_string_ext()], [wasmer_memory_grow_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param memory_name Name of the exported memory.
/// @param offset Offset to start reading.
/// @param length Number of bytes to read.
/// @return Raw vector of bytes
/// @examples
/// wasmer_memory_read_ext(ptr, "inst1", "memory", 0, 10)
/// @export
#[extendr]
pub fn wasmer_memory_read_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, memory_name: String, offset: i32, length: i32) -> Robj {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(memory) = instance.exports.get_memory(&memory_name) {
            let view = memory.view(&runtime.store);
            let mut bytes = Vec::new();
            let data_size = view.data_size().try_into().unwrap();
            for i in 0..length {
                let idx = offset + i;
                if idx >= 0 && (idx as usize) < data_size {
                    // Read byte using data_ptr
                    unsafe {
                        bytes.push(*view.data_ptr().add(idx as usize));
                    }
                }
            }
            r!(bytes)
        } else {
            r!("Memory not found")
        }
    } else {
        r!("Instance not found")
    }
}

/// Write bytes to WASM memory
///
/// @title Write WASM memory
/// @description Write bytes to WASM memory.
/// @family memory operations
/// @seealso [wasmer_memory_size_ext()], [wasmer_memory_read_ext()], [wasmer_memory_read_string_ext()], [wasmer_memory_grow_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param memory_name Name of the exported memory.
/// @param offset Offset to start writing.
/// @param bytes Raw vector of bytes to write.
/// @return TRUE if successful
/// @examples
/// wasmer_memory_write_ext(ptr, "inst1", "memory", 0, as.raw(c(1,2,3)))
/// @export
#[extendr]
pub fn wasmer_memory_write_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, memory_name: String, offset: i32, bytes: Robj) -> bool {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(memory) = instance.exports.get_memory(&memory_name) {
            let view = memory.view(&runtime.store);
            if let Some(slice) = bytes.as_raw() {
                let start = offset as usize;
                let end = start + slice.len();
                let data_size = view.data_size().try_into().unwrap();
                if end <= data_size {
                    unsafe {
                        let data_mut = view.data_ptr();
                        std::ptr::copy_nonoverlapping(slice.as_slice().as_ptr(), data_mut.add(start), slice.len());
                    }
                    return true;
                }
            }
        }
    }
    false
}

/// Read UTF-8 string from WASM memory
///
/// @title Read WASM memory as string
/// @description Read UTF-8 string from WASM memory.
/// @family memory operations
/// @seealso [wasmer_memory_size_ext()], [wasmer_memory_read_ext()], [wasmer_memory_write_ext()], [wasmer_memory_grow_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param memory_name Name of the exported memory.
/// @param offset Offset to start reading.
/// @param length Number of bytes to read.
/// @return String
/// @examples
/// wasmer_memory_read_string_ext(ptr, "inst1", "memory", 0, 10)
/// @export
#[extendr]
pub fn wasmer_memory_read_string_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, memory_name: String, offset: i32, length: i32) -> String {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(memory) = instance.exports.get_memory(&memory_name) {
            let view = memory.view(&runtime.store);
            let data_size = view.data_size().try_into().unwrap();
            let mut bytes = Vec::new();
            for i in 0..length {
                let idx = offset + i;
                if idx >= 0 && (idx as usize) < data_size {
                    unsafe {
                        bytes.push(*view.data_ptr().add(idx as usize));
                    }
                }
            }
            String::from_utf8(bytes).unwrap_or_else(|_| "".to_string())
        } else {
            "Memory not found".to_string()
        }
    } else {
        "Instance not found".to_string()
    }
}

/// Grow WASM memory by a number of pages
///
/// @title Grow WASM memory
/// @description Grow WASM memory by a number of pages.
/// @family memory operations
/// @seealso [wasmer_memory_size_ext()], [wasmer_memory_read_ext()], [wasmer_memory_write_ext()], [wasmer_memory_read_string_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param memory_name Name of the exported memory.
/// @param pages Number of pages to grow.
/// @return TRUE if successful
/// @examples
/// wasmer_memory_grow_ext(ptr, "inst1", "memory", 1)
/// @export
#[extendr]
pub fn wasmer_memory_grow_ext(mut ptr: ExternalPtr<WasmerRuntime>, instance_name: String, memory_name: String, pages: u32) -> bool {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(memory) = instance.exports.get_memory(&memory_name) {
            memory.grow(&mut runtime.store, pages).is_ok()
        } else {
            false
        }
    } else {
        false
    }
}

/// Instantiate a compiled module in the runtime, with a custom table import
///
/// @title Instantiate WASM module with table import
/// @description Instantiate a compiled WASM module in the runtime, with a custom table import.
/// @family module instantiation
/// @seealso [wasmer_instantiate_ext()], [wasmer_instantiate_with_math_imports_ext()], [wasmer_wasi_state_new_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param module_name Name of the module to instantiate.
/// @param instance_name Name to register the instance under.
/// @param table_ptr External pointer to Table to import as "env.host_table".
/// @return Status message
/// @examples
/// wasmer_instantiate_with_table_ext(ptr, "mod1", "inst1", table_ptr)
/// @export
#[extendr]
pub fn wasmer_instantiate_with_table_ext(
    mut ptr: ExternalPtr<WasmerRuntime>,
    module_name: String,
    instance_name: String,
    mut table_ptr: ExternalPtr<Table>
) -> String {
    let runtime = ptr.as_mut();
    if let Some(module) = runtime.modules.get(&module_name) {
        let import_object = imports! {
            "env" => {
                "host_table" => table_ptr.as_mut().clone(),
            }
        };
        match Instance::new(&mut runtime.store, module, &import_object) {
            Ok(final_instance) => {
                runtime.instances.insert(instance_name.clone(), final_instance.clone());
                format!("Instance '{}' created successfully with table import", instance_name)
            }
            Err(e) => format!("Error creating instance: {}", e),
        }
    } else {
        format!("Module '{}' not found", module_name)
    }
}


/// Create a new WASM Table
///
/// @title Create WASM Table
/// @description Create a new WASM Table.
/// @family table operations
/// @seealso [wasmer_table_set_ext()], [wasmer_table_grow_ext()], [wasmer_table_get_ext()], [wasmer_get_exported_table_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param min Minimum size.
/// @param max Maximum size (optional).
/// @return External pointer to Table
/// @examples
/// wasmer_table_new_ext(ptr, 1, 10)
/// @export
#[extendr]
pub fn wasmer_table_new_ext(mut ptr: ExternalPtr<WasmerRuntime>, min: u32, max: Option<u32>) -> ExternalPtr<Table> {
    let runtime = ptr.as_mut();
    let table_type = TableType::new(Type::FuncRef, min, max);
    let table = Table::new(&mut runtime.store, table_type, Value::FuncRef(None)).unwrap();
    ExternalPtr::new(table)
}

/// Set a function reference in a WASM Table
///
/// @title Set WASM Table entry
/// @description Set a function reference in a WASM Table.
/// @family table operations
/// @seealso [wasmer_table_new_ext()], [wasmer_table_grow_ext()], [wasmer_table_get_ext()], [wasmer_get_exported_table_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param table_ptr External pointer to Table.
/// @param index Index to set.
/// @param func_ptr External pointer to Function.
/// @return TRUE if successful
/// @examples
/// wasmer_table_set_ext(ptr, table_ptr, 0, func_ptr)
/// @export
#[extendr]
pub fn wasmer_table_set_ext(mut ptr: ExternalPtr<WasmerRuntime>, mut table_ptr: ExternalPtr<Table>, index: u32, mut func_ptr: ExternalPtr<Function>) -> bool {
    let runtime = ptr.as_mut();
    let table = table_ptr.as_mut();
    let func = func_ptr.as_mut();
    table.set(&mut runtime.store, index, func.clone().into()).is_ok()
}

/// Grow a WASM Table
///
/// @title Grow WASM Table
/// @description Grow a WASM Table by a number of elements.
/// @family table operations
/// @seealso [wasmer_table_new_ext()], [wasmer_table_set_ext()], [wasmer_table_get_ext()], [wasmer_get_exported_table_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param table_ptr External pointer to Table.
/// @param delta Number of elements to grow.
/// @param func_ptr External pointer to Function to fill new slots.
/// @return Previous size
/// @examples
/// wasmer_table_grow_ext(ptr, table_ptr, 1, func_ptr)
/// @export
#[extendr]
pub fn wasmer_table_grow_ext(mut ptr: ExternalPtr<WasmerRuntime>, mut table_ptr: ExternalPtr<Table>, delta: u32, mut func_ptr: ExternalPtr<Function>) -> u32 {
    let runtime = ptr.as_mut();
    let table = table_ptr.as_mut();
    let func = func_ptr.as_mut();
    table.grow(&mut runtime.store, delta, func.clone().into()).unwrap_or(0)
}

/// Get a function reference from a WASM Table
///
/// @title Get WASM Table entry
/// @description Get a function reference from a WASM Table.
/// @family table operations
/// @seealso [wasmer_table_new_ext()], [wasmer_table_set_ext()], [wasmer_table_grow_ext()], [wasmer_get_exported_table_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param table_ptr External pointer to Table.
/// @param index Index to get.
/// @return External pointer to Function (or NULL)
/// @examples
/// wasmer_table_get_ext(ptr, table_ptr, 0)
/// @export
#[extendr]
pub fn wasmer_table_get_ext(mut ptr: ExternalPtr<WasmerRuntime>, mut table_ptr: ExternalPtr<Table>, index: u32) -> Option<ExternalPtr<Function>> {
    let runtime = ptr.as_mut();
    let table = table_ptr.as_mut();
    match table.get(&mut runtime.store, index) {
        Some(Value::FuncRef(Some(f))) => Some(ExternalPtr::new(f.clone())),
        _ => None,
    }
}


/// Create a Wasmer host function from an R function with dynamic signature
///
/// @title Create dynamic R host function
/// @description Create a Wasmer host function from an R function with dynamic signature.
/// @family host function registration
/// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_i32_to_void()], [wasmer_function_new_void_to_i32()]
/// @param ptr External pointer to WasmerRuntime.
/// @param rfun R function object.
/// @param arg_types Character vector of argument types (e.g. c("i32", "f64")).
/// @param ret_types Character vector of return types (e.g. c("i32")).
/// @param _name Character string for registry name.
/// @return External pointer to Function
/// @examples
/// wasmer_function_new_ext(ptr, function(x) x, c("i32"), c("i32"), "myfun")
/// @export
#[extendr]
pub fn wasmer_function_new_ext(
    mut ptr: ExternalPtr<WasmerRuntime>,
    rfun: Robj,
    arg_types: Vec<String>,
    ret_types: Vec<String>,
    _name: String
) -> ExternalPtr<Function> {
    let runtime = ptr.as_mut();
    let id = register_r_function_internal(rfun);
    fn str_to_type(s: &str) -> Type {
        match s.to_lowercase().as_str() {
            "i32" => Type::I32,
            "i64" => Type::I64,
            "f32" => Type::F32,
            "f64" => Type::F64,
            _ => Type::I32,
        }
    }
    let param_types: Vec<Type> = arg_types.iter().map(|s| str_to_type(s)).collect();
    let result_types: Vec<Type> = ret_types.iter().map(|s| str_to_type(s)).collect();
    let fn_type = wasmer::FunctionType::new(param_types.clone(), result_types.clone());
    let fun = Function::new(
        &mut runtime.store,
        &fn_type,
        move |args: &[Value]| -> std::result::Result<Vec<Value>, wasmer::RuntimeError> {
            let r_args: Vec<Robj> = args.iter().map(|v| match v {
                Value::I32(i) => r!(*i),
                Value::I64(i) => r!(*i as i64),
                Value::F32(f) => r!(*f as f32),
                Value::F64(f) => r!(*f as f64),
                _ => r!(0),
            }).collect();
            let result = R_FUNCTION_REGISTRY.with(|reg| {
                reg.borrow().get(&id).cloned()
            }).and_then(|rfun| {
                rfun.call(pairlist!(r_args)).ok()
            });
            if let Some(r) = result {
                if result_types.len() == 0 {
                    Ok(vec![])
                } else if result_types.len() == 1 {
                    let v = match result_types[0] {
                        Type::I32 => Value::I32(r.as_integer().unwrap_or(0)),
                        Type::I64 => Value::I64(r.as_real().unwrap_or(0.0) as i64),
                        Type::F32 => Value::F32(r.as_real().unwrap_or(0.0) as f32),
                        Type::F64 => Value::F64(r.as_real().unwrap_or(0.0)),
                        _ => Value::I32(0),
                    };
                    Ok(vec![v])
                } else {
                    // Multiple return values: expect R to return a list/vector
                    let mut out = Vec::new();
                    if let Some(list) = r.as_list() {
                        for (i, ty) in result_types.iter().enumerate() {
                            let rv = list.iter().nth(i).map(|(_, v)| v).unwrap_or(r!(0));
                            let v = match ty {
                                Type::I32 => Value::I32(rv.as_integer().unwrap_or(0)),
                                Type::I64 => Value::I64(rv.as_real().unwrap_or(0.0) as i64),
                                Type::F32 => Value::F32(rv.as_real().unwrap_or(0.0) as f32),
                                Type::F64 => Value::F64(rv.as_real().unwrap_or(0.0)),
                                _ => Value::I32(0),
                            };
                            out.push(v);
                        }
                        Ok(out)
                    } else {
                        let v = match result_types[0] {
                            Type::I32 => Value::I32(r.as_integer().unwrap_or(0)),
                            Type::I64 => Value::I64(r.as_real().unwrap_or(0.0) as i64),
                            Type::F32 => Value::F32(r.as_real().unwrap_or(0.0) as f32),
                            Type::F64 => Value::F64(r.as_real().unwrap_or(0.0)),
                            _ => Value::I32(0),
                        };
                        Ok(vec![v])
                    }
                }
            } else {
                Ok(vec![Value::I32(0)])
            }
        }
    );
    ExternalPtr::new(fun)
}

/// Get a pointer to an exported table from a WASM instance by name
///
/// @title Get exported WASM Table
/// @description Get a pointer to an exported table from a WASM instance by name.
/// @family table operations
/// @seealso [wasmer_table_new_ext()], [wasmer_table_set_ext()], [wasmer_table_grow_ext()], [wasmer_table_get_ext()]
/// @param ptr External pointer to WasmerRuntime.
/// @param instance_name Name of the instance.
/// @param table_export_name Name of the exported table.
/// @return External pointer to Table, or NULL if not found
/// @examples
/// wasmer_get_exported_table_ext(ptr, "inst1", "table1")
/// @export
#[extendr]
pub fn wasmer_get_exported_table_ext(
    mut ptr: ExternalPtr<WasmerRuntime>,
    instance_name: String,
    table_export_name: String,
) -> Option<ExternalPtr<Table>> {
    let runtime = ptr.as_mut();
    if let Some(instance) = runtime.instances.get(&instance_name) {
        if let Ok(export) = instance.exports.get(&table_export_name) {
            if let wasmer::Extern::Table(table) = export {
                return Some(ExternalPtr::new(table.clone()));
            }
        }
    }
    None
}

// Macro to generate static R host function wrappers for common signatures
// Macro to generate static R host function wrappers for common signatures
macro_rules! impl_r_host_function {
    // (i32) -> i32
    ($fn_name:ident, i32, i32) => {
        /// Create a WASM host function with signature i32 -> i32
        ///
        /// @title Create host function (i32 -> i32)
        /// @description Create a WASM host function that takes an i32 and returns an i32, using an R function as the implementation.
        /// @family host function registration
        /// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_ext()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_i32_to_void()], [wasmer_function_new_void_to_i32()]
        /// @examples
        /// wasmer_function_new_i32_to_i32(ptr, function(x) x)
        /// @export
        #[extendr]
        pub fn $fn_name(
            mut ptr: ExternalPtr<WasmerRuntime>,
            rfun: Robj
        ) -> ExternalPtr<Function> {
            let runtime = ptr.as_mut();
            let id = register_r_function_internal(rfun);
            let fun = Function::new_typed(&mut runtime.store, move |x: i32| -> i32 {
                let result = R_FUNCTION_REGISTRY.with(|reg| {
                    reg.borrow().get(&id).cloned()
                }).and_then(|rfun| {
                    rfun.call(pairlist!(r!(x))).ok()
                });
                result.and_then(|r| r.as_integer()).unwrap_or(0)
            });
            ExternalPtr::new(fun)
        }
    };
    // (i32, i32) -> i32
    ($fn_name:ident, (i32, i32), i32) => {
        /// Create a WASM host function with signature (i32, i32) -> i32
        ///
        /// @title Create host function ((i32, i32) -> i32)
        /// @description Create a WASM host function that takes two i32 arguments and returns an i32, using an R function as the implementation.
        /// @family host function registration
        /// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_i32_to_void()], [wasmer_function_new_void_to_i32()]
        /// @examples
        /// wasmer_function_new_i32_i32_to_i32(ptr, function(x, y) x + y)
        /// @export
        #[extendr]
        pub fn $fn_name(
            mut ptr: ExternalPtr<WasmerRuntime>,
            rfun: Robj
        ) -> ExternalPtr<Function> {
            let runtime = ptr.as_mut();
            let id = register_r_function_internal(rfun);
            let fun = Function::new_typed(&mut runtime.store, move |x: i32, y: i32| -> i32 {
                let result = R_FUNCTION_REGISTRY.with(|reg| {
                    reg.borrow().get(&id).cloned()
                }).and_then(|rfun| {
                    rfun.call(pairlist!(r!(x), r!(y))).ok()
                });
                result.and_then(|r| r.as_integer()).unwrap_or(0)
            });
            ExternalPtr::new(fun)
        }
    };
    // (f64, f64) -> f64
    ($fn_name:ident, (f64, f64), f64) => {
        /// Create a WASM host function with signature (f64, f64) -> f64
        ///
        /// @title Create host function ((f64, f64) -> f64)
        /// @description Create a WASM host function that takes two f64 arguments and returns an f64, using an R function as the implementation.
        /// @family host function registration
        /// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_i32_to_void()], [wasmer_function_new_void_to_i32()]
        /// @examples
        /// wasmer_function_new_f64_f64_to_f64(ptr, function(x, y) x * y)
        /// @export
        #[extendr]
        pub fn $fn_name(
            mut ptr: ExternalPtr<WasmerRuntime>,
            rfun: Robj
        ) -> ExternalPtr<Function> {
            let runtime = ptr.as_mut();
            let id = register_r_function_internal(rfun);
            let fun = Function::new_typed(&mut runtime.store, move |x: f64, y: f64| -> f64 {
                let result = R_FUNCTION_REGISTRY.with(|reg| {
                    reg.borrow().get(&id).cloned()
                }).and_then(|rfun| {
                    rfun.call(pairlist!(r!(x), r!(y))).ok()
                });
                result.and_then(|r| r.as_real()).unwrap_or(0.0)
            });
            ExternalPtr::new(fun)
        }
    };
    // f64 -> f64
    ($fn_name:ident, f64, f64) => {
        /// Create a WASM host function with signature f64 -> f64
        ///
        /// @title Create host function (f64 -> f64)
        /// @description Create a WASM host function that takes an f64 and returns an f64, using an R function as the implementation.
        /// @family host function registration
        /// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_i32_to_void()], [wasmer_function_new_void_to_i32()]
        /// @examples
        /// wasmer_function_new_f64_to_f64(ptr, function(x) sqrt(x))
        /// @export
        #[extendr]
        pub fn $fn_name(
            mut ptr: ExternalPtr<WasmerRuntime>,
            rfun: Robj
        ) -> ExternalPtr<Function> {
            let runtime = ptr.as_mut();
            let id = register_r_function_internal(rfun);
            let fun = Function::new_typed(&mut runtime.store, move |x: f64| -> f64 {
                let result = R_FUNCTION_REGISTRY.with(|reg| {
                    reg.borrow().get(&id).cloned()
                }).and_then(|rfun| {
                    rfun.call(pairlist!(r!(x))).ok()
                });
                result.and_then(|r| r.as_real()).unwrap_or(0.0)
            });
            ExternalPtr::new(fun)
        }
    };
    // i32 -> ()
    ($fn_name:ident, i32, ()) => {
        /// Create a WASM host function with signature i32 -> void
        ///
        /// @title Create host function (i32 -> void)
        /// @description Create a WASM host function that takes an i32 and returns nothing, using an R function as the implementation.
        /// @family host function registration
        /// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_void_to_i32()]
        /// @examples
        /// wasmer_function_new_i32_to_void(ptr, function(x) cat(x))
        /// @export
        #[extendr]
        pub fn $fn_name(
            mut ptr: ExternalPtr<WasmerRuntime>,
            rfun: Robj
        ) -> ExternalPtr<Function> {
            let runtime = ptr.as_mut();
            let id = register_r_function_internal(rfun);
            let fun = Function::new_typed(&mut runtime.store, move |x: i32| {
                R_FUNCTION_REGISTRY.with(|reg| {
                    reg.borrow().get(&id).cloned()
                }).and_then(|rfun| {
                    rfun.call(pairlist!(r!(x))).ok()
                });
            });
            ExternalPtr::new(fun)
        }
    };
    // () -> i32
    ($fn_name:ident, (), i32) => {
        /// Create a WASM host function with signature void -> i32
        ///
        /// @title Create host function (void -> i32)
        /// @description Create a WASM host function that takes no arguments and returns an i32, using an R function as the implementation.
        /// @family host function registration
        /// @seealso [wasmer_register_r_function_ext()], [wasmer_function_new_ext()], [wasmer_function_new_i32_to_i32()], [wasmer_function_new_i32_i32_to_i32()], [wasmer_function_new_f64_f64_to_f64()], [wasmer_function_new_f64_to_f64()], [wasmer_function_new_i32_to_void()]
        /// @examples
        /// wasmer_function_new_void_to_i32(ptr, function() 42)
        /// @export
        #[extendr]
        pub fn $fn_name(
            mut ptr: ExternalPtr<WasmerRuntime>,
            rfun: Robj
        ) -> ExternalPtr<Function> {
            let runtime = ptr.as_mut();
            let id = register_r_function_internal(rfun);
            let fun = Function::new_typed(&mut runtime.store, move || -> i32 {
                let result = R_FUNCTION_REGISTRY.with(|reg| {
                    reg.borrow().get(&id).cloned()
                }).and_then(|rfun| {
                    rfun.call(pairlist!()).ok()
                });
                result.and_then(|r| r.as_integer()).unwrap_or(0)
            });
            ExternalPtr::new(fun)
        }
    };
}

impl_r_host_function!(wasmer_function_new_i32_to_i32, i32, i32);

impl_r_host_function!(wasmer_function_new_i32_i32_to_i32, (i32, i32), i32);

impl_r_host_function!(wasmer_function_new_f64_f64_to_f64, (f64, f64), f64);

impl_r_host_function!(wasmer_function_new_f64_to_f64, f64, f64);

impl_r_host_function!(wasmer_function_new_i32_to_void, i32, ());

impl_r_host_function!(wasmer_function_new_void_to_i32, (), i32);

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
    fn wasmer_list_function_signatures_ext;
    fn wasmer_memory_size_ext;
    fn wasmer_memory_read_ext;
    fn wasmer_memory_write_ext;
    fn wasmer_memory_read_string_ext;
    fn wasmer_memory_grow_ext;
    fn wasmer_table_new_ext;
    fn wasmer_table_set_ext;
    fn wasmer_table_grow_ext;
    fn wasmer_table_get_ext;
    fn wasmer_function_new_ext;
    fn wasmer_get_exported_table_ext;
    fn wasmer_function_new_i32_to_i32;
    fn wasmer_function_new_i32_i32_to_i32;
    fn wasmer_function_new_f64_f64_to_f64;
    fn wasmer_function_new_f64_to_f64;
    fn wasmer_function_new_i32_to_void;
    fn wasmer_function_new_void_to_i32;
    fn wasmer_runtime_new_with_compiler_ext;
    fn wasmer_instantiate_with_table_ext;
    fn wasmer_wasi_state_new_ext;
    fn wasmer_runtime_release_ressources;
}
