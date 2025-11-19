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
            
            "llvm" => {
                // LLVM requires LLVM 18 to be installed on the system
                // This feature is only available if compiled with --features llvm_compiler
                #[cfg(feature = "llvm_compiler")]
                {
                    match wasmer::sys::LLVM::new() {
                        Ok(llvm) => Ok(Box::new(llvm)),
                        Err(e) => Err(format!(
                            "LLVM compiler not available (requires LLVM 18 installed): {}. \
                            Try 'cranelift' or 'singlepass' instead.", 
                            e
                        )),
                    }
                }
                #[cfg(not(feature = "llvm_compiler"))]
                {
                    Err("LLVM compiler support not enabled. \
                        Rebuild the package with LLVM support if you have LLVM 18 installed. \
                        Available compilers: cranelift, singlepass".to_string())
                }
            }
            
            _ => Err(format!(
                "Unknown compiler: '{}'. Available compilers: cranelift, singlepass{}", 
                compiler_name,
                if cfg!(feature = "llvm_compiler") { ", llvm" } else { "" }
            )),
        }
    }
}
