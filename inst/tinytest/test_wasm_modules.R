# Test WebAssembly module compilation and execution
library(wasmer)

# Create runtime first
runtime <- wasmer_runtime_new()

# Test 1: Simple module compilation
simple_wat <- '
(module
  (func $add (export "add") (param $x i32) (param $y i32) (result i32)
    local.get $x
    local.get $y
    i32.add)
)
'

compile_result <- wasmer_compile_wat_ext(runtime, simple_wat, "test_module")
tinytest::expect_true(is.character(compile_result))
tinytest::expect_true(grepl("compiled successfully", compile_result, ignore.case = TRUE))

# Test 2: Module instantiation
instance_result <- wasmer_instantiate_ext(runtime, "test_module", "test_instance")
tinytest::expect_true(is.character(instance_result))
tinytest::expect_true(grepl("created successfully", instance_result, ignore.case = TRUE))

# Test 3: List exports
exports <- wasmer_list_exports_ext(runtime, "test_instance")
tinytest::expect_true(is.list(exports))
tinytest::expect_true(exports$success)
tinytest::expect_true("add" %in% exports$exports)

# Test 4: Function call
result <- wasmer_call_function_ext(runtime, "test_instance", "add", list(10L, 20L))
tinytest::expect_true(is.list(result))
tinytest::expect_true(result$success)
tinytest::expect_equal(result$values[[1]], 30)

# Test 5: Error handling for non-existent function
error_result <- wasmer_call_function_ext(runtime, "test_instance", "nonexistent", list())
tinytest::expect_true(is.list(error_result))
tinytest::expect_false(error_result$success)
tinytest::expect_true(is.character(error_result$error))

# Test 6: Error handling for non-existent instance
error_result2 <- wasmer_call_function_ext(runtime, "nonexistent_instance", "add", list(1L, 2L))
tinytest::expect_true(is.list(error_result2))
tinytest::expect_false(error_result2$success)
tinytest::expect_true(grepl("not found", error_result2$error, ignore.case = TRUE))

# Test 7: WAT to WASM binary conversion and round-trip
wat_code <- ' (module (func $double (export "double") (param $x i32) (result i32) local.get $x i32.const 2 i32.mul) ) '
wasm_bin <- wasmer_wat_to_wasm_ext(wat_code)
tinytest::expect_true(is.raw(wasm_bin))
tinytest::expect_true(length(wasm_bin) > 0)
# Optionally, compile the binary back to a module (if/when binary loader is available)
# For now, just check the binary is non-empty and valid type
