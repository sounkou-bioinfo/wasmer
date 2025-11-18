# Test basic functionality
library(wasmer)

# Create runtime
runtime <- wasmer_runtime_new()

# Test 1: Hello world example
hello_result <- wasmer_hello_world_example_ext(runtime)
tinytest::expect_true(is.character(hello_result))
tinytest::expect_true(grepl("Hello", hello_result, ignore.case = TRUE))

# Test 2: Math example
math_result <- wasmer_math_example_ext(runtime, 5L, 3L)
tinytest::expect_true(is.list(math_result))
tinytest::expect_equal(math_result$add, 8)
tinytest::expect_equal(math_result$multiply, 15)

# Test dynamic R function registration and WASM invocation (generic dispatch)
runtime <- wasmer_runtime_new()
r_double <- function(x) as.integer(x * 2)
wasmer_register_r_function_ext(runtime, "r_double", r_double)

wat_code <- '
(module
  (import "env" "r_host_call" (func $r_host_call (param i32 i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "r_double")
  (func $call_r_double (export "call_r_double") (param $x i32) (result i32)
    (i32.store (i32.const 100) (local.get $x))
    (call $r_host_call (i32.const 0) (i32.const 8) (i32.const 1) (i32.const 100))
  )
)
'

compile_result <- wasmer_compile_wat_ext(runtime, wat_code, "rhost_module")
tinytest::expect_true(is.character(compile_result))
instance_result <- wasmer_instantiate_ext(runtime, "rhost_module", "rhost_instance")
tinytest::expect_true(is.character(instance_result))
print(instance_result)
result <- wasmer_call_function_ext(runtime, "rhost_instance", "call_r_double", list(21L))
print(result) # Show the result for debugging
tinytest::expect_true(is.list(result))
tinytest::expect_true(result$success)
tinytest::expect_equal(result$values[[1]], 42)
