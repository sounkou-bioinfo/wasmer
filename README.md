
# wasmer: WebAssembly Runtime for R

<!-- badges: start -->

[![extendr](https://img.shields.io/badge/extendr-%5E0.8.1-276DC2)](https://extendr.github.io/extendr/extendr_api/)
<!-- badges: end -->

The `wasmer` package provides R bindings for the
[Wasmer](https://wasmer.io/) WebAssembly runtime, allowing you to
compile, instantiate, and execute WebAssembly modules directly from R.
This opens up possibilities for high-performance computing,
cross-language interoperability, and running untrusted code in a
sandboxed environment.

## Basic Usage

### Initialize the Runtime

``` r
library(wasmer)

# Initialize the Wasmer runtime - this must be called first
result <- wasmer_init()
result
#> [1] "Wasmer runtime initialized"
```

### Hello World Example

Let’s start with a simple example:

``` r
# Run the built-in hello world example
result <- wasmer_hello_world_example()
result
#> [1] "Hello World! Function returned: 42"
```

### Math Operations

Here’s an example with mathematical operations:

``` r
# Test basic math operations
math_result <- wasmer_math_example(5L, 3L)
math_result
#> $add
#> [1] 8
#> 
#> $multiply
#> [1] 15

# Verify the results
stopifnot(math_result$add == 8)
stopifnot(math_result$multiply == 15)
```

## Advanced Usage

### Custom WebAssembly Modules

You can compile and run custom WebAssembly modules using the WebAssembly
Text format:

``` r
# Define a Fibonacci function in WebAssembly Text Format
fibonacci_wat <- '
(module
  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (if (i32.le_s (local.get $n) (i32.const 1))
      (then (return (local.get $n)))
    )
    (return (i32.add 
      (call $fibonacci (i32.sub (local.get $n) (i32.const 1)))
      (call $fibonacci (i32.sub (local.get $n) (i32.const 2)))
    ))
  )
)
'

# Compile the WebAssembly module
compile_result <- wasmer_compile_wat(fibonacci_wat, "fibonacci_module")
compile_result
#> [1] "Module 'fibonacci_module' compiled successfully"

# Create an instance of the module
instance_result <- wasmer_instantiate("fibonacci_module", "fib_instance")
instance_result
#> [1] "Instance 'fib_instance' created successfully"
# List available exports
exports <- wasmer_list_exports("fib_instance")
exports
#> $success
#> [1] TRUE
#> 
#> $exports
#> [1] "fibonacci"
stopifnot(exports$success == TRUE)
stopifnot("fibonacci" %in% exports$exports)
```

### Calculate Fibonacci Numbers

``` r
# Test fibonacci calculations with smaller numbers due to recursive implementation
fibonacci_tests <- data.frame(
  n = 0:8,
  expected = c(0, 1, 1, 2, 3, 5, 8, 13, 21)
)

fibonacci_tests$calculated <- sapply(fibonacci_tests$n, function(n) {
  result <- wasmer_call_function_safe("fib_instance", "fibonacci", list(as.integer(n)))
  if (result$success) {
    return(result$values[[1]])
  } else {
    return(NA)
  }
})

fibonacci_tests
#>   n expected calculated
#> 1 0        0          0
#> 2 1        1          1
#> 3 2        1          1
#> 4 3        2          2
#> 5 4        3          3
#> 6 5        5          5
#> 7 6        8          8
#> 8 7       13         13
#> 9 8       21         21

# Verify all calculations are correct
all_correct <- all(fibonacci_tests$calculated == fibonacci_tests$expected, na.rm = TRUE)
stopifnot(all_correct)
```

### Host Function Integration

Wasmer allows WebAssembly modules to call functions defined in the host
environment (R/Rust):

``` r
# Example that demonstrates host function integration
host_result <- wasmer_host_function_example()
#> WASM logged: 5
host_result
#> $success
#> [1] TRUE
#> 
#> $results
#> $results$timestamp
#> [1] 1763473074
#> 
#> $results$test_result
#> [1] 25

# Verify the host function example worked
stopifnot(host_result$success == TRUE)
stopifnot(!is.null(host_result$results$test_result))
stopifnot(!is.null(host_result$results$timestamp))
```

### Complex Example: Prime Number Checker

Let’s create a more complex example that checks if a number is prime:

``` r
# Define a prime number checker in WebAssembly
prime_wat <- '
(module
  (func $is_prime (export "is_prime") (param $n i32) (result i32)
    (local $i i32)
    (local $sqrt_n i32)
    
    ;; Handle special cases
    (if (i32.lt_s (local.get $n) (i32.const 2))
      (then (return (i32.const 0))) ;; Not prime
    )
    
    (if (i32.eq (local.get $n) (i32.const 2))
      (then (return (i32.const 1))) ;; 2 is prime
    )
    
    ;; Check if even
    (if (i32.eqz (i32.rem_u (local.get $n) (i32.const 2)))
      (then (return (i32.const 0))) ;; Even numbers > 2 are not prime
    )
    
    ;; Check odd divisors up to sqrt(n)
    (local.set $i (i32.const 3))
    (local.set $sqrt_n (local.get $n)) ;; Approximate sqrt
    
    (loop $check_loop
      (if (i32.gt_s (i32.mul (local.get $i) (local.get $i)) (local.get $n))
        (then (return (i32.const 1))) ;; Prime
      )
      
      (if (i32.eqz (i32.rem_u (local.get $n) (local.get $i)))
        (then (return (i32.const 0))) ;; Not prime
      )
      
      (local.set $i (i32.add (local.get $i) (i32.const 2)))
      (br $check_loop)
    )
    
    (i32.const 1) ;; Should not reach here, but return prime
  )
)
'

# Compile and instantiate the prime checker
wasmer_compile_wat(prime_wat, "prime_module")
#> [1] "Module 'prime_module' compiled successfully"
wasmer_instantiate("prime_module", "prime_instance")
#> [1] "Instance 'prime_instance' created successfully"

# Test with known primes and non-primes
test_numbers <- c(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 21, 23, 25, 29, 31)
expected_primes <- c(2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31)

results <- sapply(test_numbers, function(n) {
  result <- wasmer_call_function_safe("prime_instance", "is_prime", list(as.integer(n)))
  return(result$values[[1]])
})

prime_results <- data.frame(
  number = test_numbers,
  is_prime = results == 1,
  expected = test_numbers %in% expected_primes
)

prime_results
#>    number is_prime expected
#> 1       2     TRUE     TRUE
#> 2       3     TRUE     TRUE
#> 3       4    FALSE    FALSE
#> 4       5     TRUE     TRUE
#> 5       6    FALSE    FALSE
#> 6       7     TRUE     TRUE
#> 7       8    FALSE    FALSE
#> 8       9    FALSE    FALSE
#> 9      10    FALSE    FALSE
#> 10     11     TRUE     TRUE
#> 11     13     TRUE     TRUE
#> 12     15    FALSE    FALSE
#> 13     17     TRUE     TRUE
#> 14     19     TRUE     TRUE
#> 15     21    FALSE    FALSE
#> 16     23     TRUE     TRUE
#> 17     25    FALSE    FALSE
#> 18     29     TRUE     TRUE
#> 19     31     TRUE     TRUE

# Verify all results are correct
all_prime_correct <- all(prime_results$is_prime == prime_results$expected)
stopifnot(all_prime_correct)
```

## Performance Comparison

Let’s compare WebAssembly performance with native R for Fibonacci
calculation:

``` r
# Native R fibonacci implementation (recursive for fair comparison)
fib_r <- function(n) {
  if (n <= 1) return(n)
  return(fib_r(n-1) + fib_r(n-2))
}

# Benchmark both implementations
n_test <- 10L


bench_results <- bench::mark(
  wasm = wasmer_call_function_safe("fib_instance", "fibonacci", list(n_test))$values[[1]],
  r = fib_r(n_test),
  check = FALSE,
  min_iterations = 5
)

bench_results
#> # A tibble: 2 × 6
#>   expression      min   median `itr/sec` mem_alloc `gc/sec`
#>   <bch:expr> <bch:tm> <bch:tm>     <dbl> <bch:byt>    <dbl>
#> 1 wasm         1.97µs   2.16µs   435192.        0B      0  
#> 2 r           26.27µs  27.32µs    36078.    23.1KB     36.1

# Extract results for verification
wasm_result <- bench_results$result[[1]]
r_result <- bench_results$result[[2]]
paste0("Fibonacci(", n_test, "):")
#> [1] "Fibonacci(10):"
paste0("WebAssembly result:", wasm_result)
#> [1] "WebAssembly result:"
paste0("R result:", r_result)
#> [1] "R result:"

# Verify results match
stopifnot(wasm_result == r_result)
```

## Running Tests

This README serves as both documentation and a test suite. To run the
tests:

``` r
# The chunks above serve as integration tests
# For additional unit tests, run:
tinytest::test_all(".")
```

## LLM Usage Disclosure

Code and documentation in this project have been generated with the
assistance of the github Copilot LLM tools. While we have reviewed and
edited the generated content, we acknowledge that LLM tools were used in
the creation process and accordingly (since these models are trained on
GPL code and other commons + proprietary software license is fake
anyway) the code is released under GPL-3. So if you use this code in any
way, you must comply with the GPL-3 license.
