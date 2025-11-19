use wasmer::sys::CompilerConfig;

pub struct CompilerUtils;

impl CompilerUtils {
    pub fn get_compiler_config(compiler_name: &str) -> std::result::Result<Box<dyn CompilerConfig>, String> {
        match compiler_name.to_lowercase().as_str() {
            "cranelift" => {
                Ok(Box::new(wasmer::sys::Cranelift::default()))
            }

            "singlepass" => {
                Ok(Box::new(wasmer::sys::Singlepass::default()))
            }
            _ => Err(format!("Unknown compiler: {}", compiler_name)),
        }
    }
}
