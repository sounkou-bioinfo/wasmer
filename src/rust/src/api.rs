use extendr_api::prelude::*;
use wasmer::{Function, Store};

/// Create a generic host function for WASM that dispatches to registered R functions
pub fn create_generic_r_host_function(store: &mut Store) -> Function {
    Function::new_typed(store, |_name_ptr: i32, _name_len: i32, _arg: i32| -> i32 {
        // This is a stub: in a real implementation, you would read the function name and argument from WASM memory
        // and dispatch to the registered R function, returning the result as i32 (or other type as needed).
        // For demo, just return 42.
        42
    })
}

extendr_module! {
    mod wasmer_api;
}
