use extendr_api::prelude::*;
use wasmer::{Value, Type};

/// Utilities for converting between R and WebAssembly types
pub struct TypeConverter;

impl TypeConverter {
    /// Convert R value to WebAssembly Value
    pub fn r_to_wasm(r_value: &Robj) -> std::result::Result<Value, String> {
        match r_value.rtype() {
            Rtype::Integers => {
                if let Some(val) = r_value.as_integer() {
                    Ok(Value::I32(val))
                } else {
                    Err("Could not convert R integer to i32".to_string())
                }
            }
            Rtype::Doubles => {
                if let Some(val) = r_value.as_real() {
                    Ok(Value::F64(val))
                } else {
                    Err("Could not convert R real to f64".to_string())
                }
            }
            Rtype::Logicals => {
                if let Some(val) = r_value.as_logical() {
                    Ok(Value::I32(if val.is_true() { 1 } else { 0 }))
                } else {
                    Err("Could not convert R logical to i32".to_string())
                }
            }
            _ => Err(format!("Unsupported R type: {:?}", r_value.rtype())),
        }
    }

    /// Convert WebAssembly Value to R value
    pub fn wasm_to_r(wasm_value: &Value) -> Robj {
        match wasm_value {
            Value::I32(i) => r!(*i),
            Value::I64(i) => r!(*i as f64), // R doesn't have i64, use f64
            Value::F32(f) => r!(*f as f64),
            Value::F64(f) => r!(*f),
            Value::V128(_) => r!(NA_REAL), // Not supported
            Value::FuncRef(_) => r!(NA_REAL), // Not supported
            Value::ExternRef(_) => r!(NA_REAL), // Not supported
            Value::ExceptionRef(_) => r!(NA_REAL), // Not supported
        }
    }

    /// Convert R vector to WebAssembly Value vector
    pub fn r_vector_to_wasm(r_values: List) -> std::result::Result<Vec<Value>, String> {
        let mut wasm_values = Vec::new();
        
        for (_name, value) in r_values.iter() {
            match Self::r_to_wasm(&value) {
                Ok(wasm_val) => wasm_values.push(wasm_val),
                Err(e) => return Err(e),
            }
        }
        
        Ok(wasm_values)
    }

    /// Convert WebAssembly Value vector to R list
    pub fn wasm_vector_to_r(wasm_values: &[Value]) -> Robj {
        let r_values: Vec<Robj> = wasm_values.iter()
            .map(|v| Self::wasm_to_r(v))
            .collect();
        
        // Convert to a simple vector if single value, otherwise return a list
        if r_values.len() == 1 {
            r_values[0].clone()
        } else {
            // Try to create a list, fall back to NA if it fails
            List::from_values(r_values).into()
        }
    }

    /// Get WebAssembly type from R value
    pub fn r_to_wasm_type(r_value: &Robj) -> std::result::Result<Type, String> {
        match r_value.rtype() {
            Rtype::Integers => Ok(Type::I32),
            Rtype::Doubles => Ok(Type::F64),
            Rtype::Logicals => Ok(Type::I32),
            _ => Err(format!("Unsupported R type for WASM: {:?}", r_value.rtype())),
        }
    }

    /// Validate that R arguments match expected WebAssembly function signature
    pub fn validate_args(r_args: &List, expected_types: &[Type]) -> std::result::Result<(), String> {
        if r_args.len() != expected_types.len() {
            return Err(format!(
                "Argument count mismatch: expected {}, got {}",
                expected_types.len(),
                r_args.len()
            ));
        }

        for (i, (_name, arg)) in r_args.iter().enumerate() {
            let r_type = Self::r_to_wasm_type(&arg)?;
            if r_type != expected_types[i] {
                return Err(format!(
                    "Argument type mismatch at position {}: expected {:?}, got {:?}",
                    i, expected_types[i], r_type
                ));
            }
        }

        Ok(())
    }
}
