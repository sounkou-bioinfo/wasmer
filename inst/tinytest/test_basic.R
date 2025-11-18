# Test basic functionality
library(wasmer)

# Test 1: Runtime initialization
init_result <- wasmer_init()
tinytest::expect_true(is.character(init_result))
tinytest::expect_true(grepl("initialized", init_result, ignore.case = TRUE))

# Test 2: Hello world example
hello_result <- wasmer_hello_world_example()
tinytest::expect_true(is.character(hello_result))
tinytest::expect_true(grepl("Hello", hello_result, ignore.case = TRUE))

# Test 3: Math example
math_result <- wasmer_math_example(5L, 3L)
tinytest::expect_true(is.list(math_result))
tinytest::expect_equal(math_result$add, 8)
tinytest::expect_equal(math_result$multiply, 15)

# Test 4: Host function example
host_result <- wasmer_host_function_example()
tinytest::expect_true(is.list(host_result))
tinytest::expect_true(host_result$success)
tinytest::expect_true(is.list(host_result$results))
