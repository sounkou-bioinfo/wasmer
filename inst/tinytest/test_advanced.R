# Test advanced WebAssembly features
library(wasmer)

# Create runtime
runtime <- wasmer_runtime_new()

# Test 1: Fibonacci module
fibonacci_wat <- '
(module
  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (local $a i32)
    (local $b i32)
    (local $i i32)

    (if (i32.le_s (local.get $n) (i32.const 1))
      (then (return (local.get $n)))
    )

    (local.set $a (i32.const 0))
    (local.set $b (i32.const 1))
    (local.set $i (i32.const 2))

    (loop $fib_loop
      (local.set $a
        (i32.add (local.get $a) (local.get $b)))
      (local.set $b
        (i32.sub (local.get $a) (local.get $b)))
      (local.set $i
        (i32.add (local.get $i) (i32.const 1)))

      (br_if $fib_loop
        (i32.lt_s (local.get $i) (local.get $n)))
    )

    (local.get $a)
  )
)
'

wasmer_compile_wat_ext(runtime, fibonacci_wat, "fib_module")
wasmer_instantiate_ext(runtime, "fib_module", "fib_instance")

# Test fibonacci sequence
fibonacci_expected <- c(0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55)
for (i in 0:10) {
    result <- wasmer_call_function_ext(runtime, "fib_instance", "fibonacci", list(as.integer(i)))
    tinytest::expect_true(result$success, info = paste("Fibonacci", i, "should succeed"))
    tinytest::expect_equal(result$values[[1]], fibonacci_expected[i + 1],
        info = paste("Fibonacci", i, "should equal", fibonacci_expected[i + 1])
    )
}

# Test 2: Multiple parameters and complex logic
complex_wat <- '
(module
  (func $gcd (export "gcd") (param $a i32) (param $b i32) (result i32)
    (local $temp i32)

    (loop $gcd_loop
      (if (i32.eqz (local.get $b))
        (then (return (local.get $a)))
      )

      (local.set $temp (i32.rem_u (local.get $a) (local.get $b)))
      (local.set $a (local.get $b))
      (local.set $b (local.get $temp))

      (br $gcd_loop)
    )

    (local.get $a)
  )

  (func $lcm (export "lcm") (param $a i32) (param $b i32) (result i32)
    (i32.div_u
      (i32.mul (local.get $a) (local.get $b))
      (call $gcd (local.get $a) (local.get $b))
    )
  )
)
'

wasmer_compile_wat_ext(runtime, complex_wat, "math_module")
wasmer_instantiate_ext(runtime, "math_module", "math_instance")

# Test GCD function
gcd_result <- wasmer_call_function_ext(runtime, "math_instance", "gcd", list(48L, 18L))
tinytest::expect_true(gcd_result$success)
tinytest::expect_equal(gcd_result$values[[1]], 6) # GCD(48, 18) = 6

# Test LCM function
lcm_result <- wasmer_call_function_ext(runtime, "math_instance", "lcm", list(12L, 8L))
tinytest::expect_true(lcm_result$success)
tinytest::expect_equal(lcm_result$values[[1]], 24) # LCM(12, 8) = 24

# Test 3: Edge cases
edge_cases <- list(
    list(a = 0L, b = 5L, gcd_expected = 5L),
    list(a = 7L, b = 7L, gcd_expected = 7L),
    list(a = 1L, b = 100L, gcd_expected = 1L)
)

for (case in edge_cases) {
    result <- wasmer_call_function_ext(runtime, "math_instance", "gcd", list(case$a, case$b))
    tinytest::expect_true(result$success,
        info = paste("GCD(", case$a, ",", case$b, ") should succeed")
    )
    tinytest::expect_equal(result$values[[1]], case$gcd_expected,
        info = paste("GCD(", case$a, ",", case$b, ") should equal", case$gcd_expected)
    )
}
