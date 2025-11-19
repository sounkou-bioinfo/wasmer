
# wasmer: WebAssembly Runtime for R

<!-- badges: start -->

[![extendr](https://img.shields.io/badge/extendr-%5E0.8.1-276DC2)](https://extendr.github.io/extendr/extendr_api/)
<!-- badges: end -->

The `wasmer` package provides R bindings for the `rust` based
[Wasmer](https://github.com/wasmerio/wasmer) WebAssembly runtime,
allowing you to compile, instantiate, and execute WebAssembly modules
directly from R. This opens up possibilities for high-performance
computing, cross-language interoperability, and running untrusted code
in a sandboxed environment. This is a wip.

## Installation

This package uses {rextenr}, so you will need a rust installation

``` r
remotes::install_github("sounkou-bioinfo/wasmer")
```

## Usage

### Initialize the Runtime

``` r
library(wasmer)

# Create the Wasmer runtime (must be called first)
runtime <- wasmer_runtime_new()
runtime
#> <pointer: 0x572b72c6dd80>
```

### Compiler Selection

You can choose the compiler backend when initializing the runtime. Each
compiler offers different trade-offs:

- **Cranelift** (default): Fast compilation with good runtime
  performance
- **Singlepass**: Fastest compilation (linear time), ideal for
  blockchains/gas metering
- **LLVM** (optional): Best runtime performance (~50% faster than
  Cranelift), but requires LLVM 18 installed

``` r
# Initialize with Cranelift (default)
rt_cranelift <- wasmer_runtime_new_with_compiler_ext("cranelift")

# Initialize with Singlepass for fastest compilation
rt_singlepass <- wasmer_runtime_new_with_compiler_ext("singlepass")

# LLVM is available only if package was built with LLVM support
# (requires LLVM 18 on system)
# rt_llvm <- wasmer_runtime_new_with_compiler_ext("llvm")
```

### WASI Support

The package supports WASI (WebAssembly System Interface), allowing WASM
modules to interact with system resources in a sandboxed way.

``` r
# Create a WASI-enabled runtime
rt_wasi <- wasmer_runtime_new()

# Create WASI environment (must be done before instantiation)
wasmer_wasi_state_new_ext(rt_wasi, "hello_wasi")
#> [1] TRUE

# Simple WASI "Hello World" that prints to stdout
# Note: WASI modules use wasi_snapshot_preview1 imports
hello_wasi_wat <- '
(module
  ;; Import WASI fd_write function for writing to stdout
  (import "wasi_snapshot_preview1" "fd_write" 
    (func $fd_write (param i32 i32 i32 i32) (result i32)))
  
  (memory (export "memory") 1)
  (data (i32.const 0) "Hello from WASI!\\0a")
  
  ;; _start is the default WASI entry point
  (func $_start (export "_start")
    ;; Write the iovec structure at address 100
    ;; iovec:  [ptr, len]
    (i32.store (i32.const 100) (i32.const 0))  ;; ptr to string
    (i32.store (i32.const 104) (i32.const 17)) ;; length of string
    
    ;; Call fd_write(1, 100, 1, 200)
    ;; fd=1 (stdout), iovs=100, iovs_len=1, nwritten=200
    (drop (call $fd_write
      (i32.const 1)   ;; file descriptor 1 = stdout
      (i32.const 100) ;; pointer to iovec array
      (i32.const 1)   ;; number of iovecs
      (i32.const 200) ;; pointer to nwritten result
    ))
  )
)
'

wasmer_compile_wat_ext(rt_wasi, hello_wasi_wat, "hello_wasi")
#> [1] "Module 'hello_wasi' compiled successfully"
wasmer_instantiate_ext(rt_wasi, "hello_wasi", "wasi_instance")
#> [1] "Instance 'wasi_instance' created successfully"

# Call the WASI _start function (prints "Hello from WASI!")
# Note: WASI captured output is currently not exposed in R bindings
# But the module executes successfully
result <- wasmer_call_function_ext(rt_wasi, "wasi_instance", "_start", list())
result$success
#> [1] TRUE
```

### Math Operations compiled from Rust

``` r
# Test basic math operations
math_result <- wasmer_math_example_ext(runtime, 5L, 3L)
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

### Custom WebAssembly Modules

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
compile_result <- wasmer_compile_wat_ext(runtime, fibonacci_wat, "fibonacci_module")
compile_result
#> [1] "Module 'fibonacci_module' compiled successfully"

# Create an instance of the module
instance_result <- wasmer_instantiate_ext(runtime, "fibonacci_module", "fib_instance")
instance_result
#> [1] "Instance 'fib_instance' created successfully"
# List available exports
exports <- wasmer_list_exports_ext(runtime, "fib_instance")
exports
#> $success
#> [1] TRUE
#> 
#> $exports
#> [1] "fibonacci"
stopifnot(exports$success == TRUE)
stopifnot("fibonacci" %in% exports$exports)

# Native R fibonacci implementation 
fib_r <- function(n) {
  if (n <= 1) return(n)
  return(fib_r(n-1) + fib_r(n-2))
}
# Loop version of R



# Benchmark both implementations
n_test <- 10L

bench_results <- bench::mark(
  wasm = wasmer_call_function_ext(runtime, "fib_instance", "fibonacci", list(n_test))$values[[1]],
  r = fib_r(n_test),
  check = FALSE,
  min_iterations = 5
)

bench_results
#> # A tibble: 2 × 6
#>   expression      min   median `itr/sec` mem_alloc `gc/sec`
#>   <bch:expr> <bch:tm> <bch:tm>     <dbl> <bch:byt>    <dbl>
#> 1 wasm         2.12µs    2.3µs   350736.        0B     35.1
#> 2 r           26.25µs   28.4µs    34731.    32.7KB     31.3
stopifnot(bench_results$wasm[[1]] == bench_results$r[[1]])
```

### Calculate Fibonacci Numbers

``` r
# Test fibonacci calculations with smaller numbers due to recursive implementation
fibonacci_tests <- data.frame(
  n = 0:8,
  expected = c(0, 1, 1, 2, 3, 5, 8, 13, 21)
)

fibonacci_tests$calculated <- sapply(fibonacci_tests$n, function(n) {
  result <- wasmer_call_function_ext(runtime, "fib_instance", "fibonacci", list(as.integer(n)))
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

### Prime Number Checker

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
wasmer_compile_wat_ext(runtime, prime_wat, "prime_module")
#> [1] "Module 'prime_module' compiled successfully"
wasmer_instantiate_ext(runtime, "prime_module", "prime_instance")
#> [1] "Instance 'prime_instance' created successfully"

# Test with known primes and non-primes
test_numbers <- c(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 21, 23, 25, 29, 31)
expected_primes <- c(2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31)

results <- sapply(test_numbers, function(n) {
  result <- wasmer_call_function_ext(runtime, "prime_instance", "is_prime", list(as.integer(n)))
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

## Inspect Exported Function Signatures

``` r
# After instantiating a module, you can inspect its exported function signatures
signatures <- wasmer_list_function_signatures_ext(runtime, "fib_instance")
print(signatures)
#> $name
#> [1] "fibonacci"
#> 
#> $params
#> [1] "[I32]"
#> 
#> $results
#> [1] "[I32]"

# You can also inspect other instances
signatures_prime <- wasmer_list_function_signatures_ext(runtime, "prime_instance")
print(signatures_prime)
#> $name
#> [1] "is_prime"
#> 
#> $params
#> [1] "[I32]"
#> 
#> $results
#> [1] "[I32]"
```

## WAT to WASM Binary Conversion and Binary Module Loading

``` r
# Convert WAT to WASM binary
wat_code <- ' (module (func $double (export "double") (param $x i32) (result i32) local.get $x i32.const 2 i32.mul) ) '
wasm_bin <- wasmer_wat_to_wasm_ext(wat_code)
stopifnot(is.raw(wasm_bin))
stopifnot(length(wasm_bin) > 0)

# Compile the binary WASM into a module
compile_bin_result <- wasmer_compile_wasm_ext(runtime, wasm_bin, "double_module_bin")
stopifnot(is.character(compile_bin_result))
stopifnot(grepl("compiled from binary successfully", compile_bin_result, ignore.case = TRUE))

# Instantiate and call the exported function
instance_bin_result <- wasmer_instantiate_ext(runtime, "double_module_bin", "double_instance_bin")
stopifnot(is.character(instance_bin_result))
stopifnot(grepl("created successfully", instance_bin_result, ignore.case = TRUE))

# Call the function
result_bin <- wasmer_call_function_ext(runtime, "double_instance_bin", "double", list(21L))
stopifnot(result_bin$success)
stopifnot(result_bin$values[[1]] == 42)
```

## Register and Call an R Function from WASM

``` r
# Define an R function to double a value
r_double <- function(x) as.integer(x * 2)
wasmer_register_r_function_ext(runtime, "r_double", r_double)
#> [1] TRUE

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
compile_result
#> [1] "Module 'rhost_module' compiled successfully"
result <- wasmer_call_function_ext(runtime, "rhost_instance", "call_r_double", list(21L))
stopifnot(result$values[[1]] == 42)
```

## Simple WASM Memory Example

This example demonstrates how to use Wasmer’s memory utilities from R.

``` r
# Create a runtime
rt <- wasmer_runtime_new()

# Compile a simple WAT module with exported memory
wat <- '(module (memory (export "memory") 1) (func (export "write") (param i32 i32) (result i32)
  local.get 0
  local.get 1
  i32.store
  local.get 0))'
wasmer_compile_wat_ext(rt, wat, "memmod")
#> [1] "Module 'memmod' compiled successfully"
wasmer_instantiate_ext(rt, "memmod", "inst")
#> [1] "Instance 'inst' created successfully"

# Write bytes to memory
wasmer_memory_write_ext(rt, "inst", "memory", 0, as.raw(c(65, 66, 67))) # Write 'A', 'B', 'C' at offset 0
#> [1] TRUE

# Read bytes from memory
bytes <- wasmer_memory_read_ext(rt, "inst", "memory", 0, 3)
print(bytes) # Should print raw vector: 41 42 43
#> [1] 41 42 43

# Grow memory by 1 page (64KiB)
success <- wasmer_memory_grow_ext(rt, "inst", "memory", 1L)
print(success) # Should print TRUE if memory was grown
#> [1] TRUE

# Read as string
str <- wasmer_memory_read_string_ext(rt, "inst", "memory", 0, 3)
print(str) # Should print "ABC"
#> [1] "ABC"
```

This example shows how to write and read bytes and strings from WASM
memory using Wasmer’s R interface.

## WASM Tables and Typed Host Functions

This example demonstrates using typed host functions that can be
inserted into WASM tables.

``` r
# Create runtime
rt <- wasmer_runtime_new()

# Example 1: Typed host function for (i32, i32) -> i32
host_add <- function(x, y) as.integer(x + y)
host_multiply <- function(x, y) as.integer(x * y)

# Create typed functions (can be used in tables/funcref)
add_func <- wasmer_function_new_i32_i32_to_i32(rt, host_add)
mul_func <- wasmer_function_new_i32_i32_to_i32(rt, host_multiply)

# Create WASM table
table_ptr <- wasmer_table_new_ext(rt, 2L, 10L)

# Insert host functions into table
wasmer_table_set_ext(rt, table_ptr, 0L, add_func)
#> [1] TRUE
wasmer_table_set_ext(rt, table_ptr, 1L, mul_func)
#> [1] TRUE

# Example 2: Single argument typed functions
host_square <- function(x) as.integer(x * x)
host_sqrt <- function(x) sqrt(x)

# i32 -> i32 function
square_func <- wasmer_function_new_i32_to_i32(rt, host_square)

# f64 -> f64 function
sqrt_func <- wasmer_function_new_f64_to_f64(rt, host_sqrt)

# Example 3: Logging function (i32 -> void)
host_log <- function(x) {
  cat("WASM logged value:", x, "\n")
}
log_func <- wasmer_function_new_i32_to_void(rt, host_log)

# Example 4: Generator function (() -> i32)
counter <- 0
host_next <- function() {
  counter <<- counter + 1
  as.integer(counter)
}
next_func <- wasmer_function_new_void_to_i32(rt, host_next)

# Demonstrate usage with a WASM module
wat_with_imports <- '
(module
  (import "env" "host_add" (func $host_add (param i32 i32) (result i32)))
  (import "env" "host_square" (func $host_square (param i32) (result i32)))
  
  (func (export "test_imports") (param $x i32) (param $y i32) (result i32)
    ;; Call host_add(x, y) then host_square the result
    (call $host_square (call $host_add (local.get $x) (local.get $y)))
  )
)
'

wasmer_compile_wat_ext(rt, wat_with_imports, "imports_mod")
#> [1] "Module 'imports_mod' compiled successfully"
wasmer_instantiate_with_math_imports_ext(rt, "imports_mod", "imports_inst")
#> [1] "Error creating instance: Error while importing \"env\".\"host_add\": unknown import. Expected Function(FunctionType { params: [I32, I32], results: [I32] })"

# Note: The above will fail because we need to properly wire imports
# For now, the examples show the available typed function signatures
```

### Available Typed Host Function Signatures

The package provides the following typed host function creators:

- `wasmer_function_new_i32_to_i32(rt, rfun)` - (i32) -\> i32
- `wasmer_function_new_i32_i32_to_i32(rt, rfun)` - (i32, i32) -\> i32  
- `wasmer_function_new_f64_f64_to_f64(rt, rfun)` - (f64, f64) -\> f64
- `wasmer_function_new_f64_to_f64(rt, rfun)` - (f64) -\> f64
- `wasmer_function_new_i32_to_void(rt, rfun)` - (i32) -\> void (for
  logging)
- `wasmer_function_new_void_to_i32(rt, rfun)` - () -\> i32 (for
  generators)

> \[!IMPORTANT\] **Typed vs Dynamic Host Functions**
>
> - **Typed Host Functions**: Created with `wasmer_function_new_*`
>   variants. These have fixed signatures and **CAN** be inserted into
>   WASM tables and used as `funcref`.
> - **Dynamic Host Functions**: Created with `wasmer_function_new_ext`.
>   These are flexible with any signature, but **CANNOT** be used in
>   tables due to Wasmer engine limitations.
>
> Use typed functions when you need table/funcref support, dynamic
> functions for direct imports only.

## LLM Usage Disclosure

Code and documentation in this project have been generated with the
assistance of the github Copilot LLM tools. While we have reviewed and
edited the generated content, we acknowledge that LLM tools were used in
the creation process and accordingly (since these models are trained on
GPL code and other commons + proprietary software license is fake
anyway) the code is released under GPL-3. So if you use this code in any
way, you must comply with the GPL-3 license.
