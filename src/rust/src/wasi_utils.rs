use std::sync::Arc;
use wasmer::Store;
use wasmer_wasix::WasiEnv;

pub struct WasiUtils;

impl WasiUtils {
    /// Create a WASI environment with default settings (capturing stdout/stderr)
    pub fn create_wasi_env(store: &mut Store, module_name: &str) -> std::result::Result<wasmer_wasix::WasiFunctionEnv, String> {
        // Create a pluggable runtime with the tokio task manager
        let runtime = Arc::new(wasmer_wasix::PluggableRuntime::new(
            Arc::new(wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager::new(
                tokio::runtime::Handle::current()
            ))
        ));
        
        // Create and configure the builder
        let mut builder = WasiEnv::builder(module_name);
        builder = builder.stdout(Box::new(wasmer_wasix::Pipe::new()));
        builder = builder.stderr(Box::new(wasmer_wasix::Pipe::new()));
        
        // Set the runtime (modifies builder in-place)
        builder.set_runtime(runtime);
        
        // Finalize the builder
        match builder.finalize(store) {
            Ok(env) => Ok(env),
            Err(e) => Err(format!("Failed to create WASI state: {}", e)),
        }
    }

    // Note: Reading stdout/stderr from WasiEnv is tricky because of ownership and locking.
    // We might need to access the pipes directly if we kept them, or use the fs state.
    // For now, let's try to access via state().
}
