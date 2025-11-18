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

# Test 3: Host function example
# (If exported, otherwise comment out)
# host_result <- wasmer_host_function_example_ext(runtime)
# tinytest::expect_true(is.list(host_result))
# tinytest::expect_true(host_result$success)
# tinytest::expect_true(is.list(host_result$results))
