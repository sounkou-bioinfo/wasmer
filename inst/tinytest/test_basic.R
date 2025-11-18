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

# Test dynamic R function registration and WASM invocation
runtime <- wasmer_runtime_new()
r_double <- function(x) x * 2
wasmer_register_r_function_ext(runtime, "r_double", r_double)

wat_code <- '
(module
  (import "env" "r_double" (func $r_double (param i32) (result i32)))
  (func $call_r_double (export "call_r_double") (param $x i32) (result i32)
    (call $r_double (local.get $x))
  )
)
'

compile_result <- wasmer_compile_wat_ext(runtime, wat_code, "rhost_module")
tinytest::expect_true(is.character(compile_result))
instance_result <- wasmer_instantiate_ext(runtime, "rhost_module", "rhost_instance")
tinytest::expect_true(is.character(instance_result))

result <- wasmer_call_function_ext(runtime, "rhost_instance", "call_r_double", list(21L))
tinytest::expect_true(is.list(result))
tinytest::expect_true(result$success)
tinytest::expect_equal(result$values[[1]], 42)
