use extendr_api::prelude::*;
use wasmer::{Store, Function};

/// Advanced Wasmer utilities for host function imports
pub struct WasmerHostFunctions;

impl WasmerHostFunctions {
    /// Create a simple logging function that can be imported by WASM modules
    pub fn create_log_function(store: &mut Store) -> Function {
        Function::new_typed(store, |x: i32| {
            rprintln!("WASM Log: {}", x);
        })
    }

    /// Create a function that returns the current timestamp
    pub fn create_timestamp_function(store: &mut Store) -> Function {
        Function::new_typed(store, || -> i64 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        })
    }

    /// Create a random number generator function
    pub fn create_random_function(store: &mut Store) -> Function {
        Function::new_typed(store, || -> i32 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().hash(&mut hasher);
            (hasher.finish() % 1000) as i32
        })
    }

    /// Create math functions that can be imported
    pub fn create_math_functions(store: &mut Store) -> std::collections::HashMap<String, Function> {
        let mut functions = std::collections::HashMap::new();

        // Square function
        functions.insert(
            "square".to_string(),
            Function::new_typed(store, |x: i32| -> i32 { x * x })
        );

        // Cube function
        functions.insert(
            "cube".to_string(),
            Function::new_typed(store, |x: i32| -> i32 { x * x * x })
        );

        // Factorial (limited to prevent overflow)
        functions.insert(
            "factorial".to_string(),
            Function::new_typed(store, |x: i32| -> i32 {
                if x <= 0 { 1 }
                else if x > 10 { -1 } // Error indicator for too large numbers
                else { (1..=x).product() }
            })
        );

        functions
    }

    /// Create a generic host function dispatcher for dynamic R function calls from WASM
    #[allow(dead_code)]
    pub fn create_generic_r_host_function(store: &mut Store) -> Function {
        Function::new_typed(store, |_name_ptr: i32, _name_len: i32, _arg: i32| -> i32 {
            // Placeholder for dynamic R host function call
            0
        })
    }
}
