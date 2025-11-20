
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
#> <pointer: 0x5f15009a54a0>
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

# Native R fibonacci implementations
# Naive recursive (grows stack)
fib_r <- function(n) {
  if (n <= 1) return(n)
  return(fib_r(n-1) + fib_r(n-2))
}

# Tail-recursive using Tailcall (R >= 4.6.0, doesn't grow stack)
fib_tailcall <- function(n) {
  fib_iter <- function(a, b, count) {
    if (count == 0) {
      a
    } else {
      # Force evaluation to avoid deferred computations
      new_a <- b
      new_b <- a + b
      Tailcall(fib_iter, new_a, new_b, count - 1)
    }
  }
  fib_iter(0L, 1L, n)
}

# Benchmark all three implementations
n_test <- 20L

bench_results <- bench::mark(
  wasm = wasmer_call_function_ext(runtime, "fib_instance", "fibonacci", list(n_test))$values[[1]],
  r_naive = fib_r(n_test),
  r_tailcall = fib_tailcall(n_test),
  check = FALSE,
  min_iterations = 10
)

bench_results
#> # A tibble: 3 × 6
#>   expression      min   median `itr/sec` mem_alloc `gc/sec`
#>   <bch:expr> <bch:tm> <bch:tm>     <dbl> <bch:byt>    <dbl>
#> 1 wasm        26.13µs  27.43µs    36233.        0B      0  
#> 2 r_naive      3.41ms   3.44ms      289.    32.7KB     34.9
#> 3 r_tailcall  10.63µs  11.38µs    84490.        0B     42.3
stopifnot(bench_results$wasm[[1]] == bench_results$r_naive[[1]])
stopifnot(bench_results$wasm[[1]] == bench_results$r_tailcall[[1]])
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
result_bin$values[[1]]
#> [1] 42
```

## Register and Call an R Function from WASM

``` r


## Corrected Example: Register and Call an R Function from WASM

# Define an R function to double a value
r_double <- function(a) {
  2L * as.integer(a)
}
handle <- wasmer_register_r_function_ext(runtime, "r_double", r_double)

# The Rust host function expects (handle, args_ptr, argc), but only the args are passed to R
wat_code <- sprintf('
(module
  (import "env" "r_host_call" (func $r_host_call (param i32 i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "r_double")
  (func $call_r_double (export "call_r_double") (param $x i32) (result i32)
    (i32.store (i32.const 100) (local.get $x))
    (call $r_host_call (i32.const %d) (i32.const 100) (i32.const 1))
  )
)
', handle)

compile_result <- wasmer_compile_wat_ext(runtime, wat_code, "rhost_module")
compile_result
#> [1] "Module 'rhost_module' compiled successfully"
instance_result <- wasmer_instantiate_ext(runtime, "rhost_module", "rhost_instance")
instance_result
#> [1] "Instance 'rhost_instance' created successfully"
result <- wasmer_call_function_ext(runtime, "rhost_instance", "call_r_double", list(21L))
#> [wasmer] Host call: handle=1, args=[21]
#> [wasmer] Found R function for handle 1, calling...
#> [wasmer] R call result: 42
#> [wasmer] Returning integer value: 42
result
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 42
stopifnot(result$values[[1]] == 42)
```

## Simple WASM Memory Example

This example demonstrates how to use Wasmer’s memory utilities from R.

``` r
# Compile a simple WAT module with exported memory
wat <- '(module (memory (export "memory") 1) (func (export "write") (param i32 i32) (result i32)
  local.get 0
  local.get 1
  i32.store
  local.get 0))'
wasmer_compile_wat_ext(runtime, wat, "memmod")
#> [1] "Module 'memmod' compiled successfully"
wasmer_instantiate_ext(runtime, "memmod", "inst")
#> [1] "Instance 'inst' created successfully"

# Write bytes to memory
wasmer_memory_write_ext(runtime, "inst", "memory", 0, as.raw(c(65, 66, 67))) # Write 'A', 'B', 'C' at offset 0
#> [1] TRUE

# Read bytes from memory
bytes <- wasmer_memory_read_ext(runtime, "inst", "memory", 0, 3)
print(bytes) # Should print raw vector: 41 42 43
#> [1] 41 42 43

# Grow memory by 1 page (64KiB)
success <- wasmer_memory_grow_ext(runtime, "inst", "memory", 1L)
print(success) # Should print TRUE if memory was grown
#> [1] TRUE

# Read as string
str <- wasmer_memory_read_string_ext(runtime, "inst", "memory", 0, 3)
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
rt <- runtime

# Example 1: (i32, i32) -> i32 - Binary operations
host_add <- function(x, y) as.integer(x + y)
host_multiply <- function(x, y) as.integer(x * y)

add_func <- wasmer_function_new_i32_i32_to_i32(runtime, host_add)
mul_func <- wasmer_function_new_i32_i32_to_i32(runtime, host_multiply)

# Create and use in WASM table
table_ptr <- wasmer_table_new_ext(runtime, 2L, 10L)
wasmer_table_set_ext(runtime, table_ptr, 0L, add_func)
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 1L, mul_func)
#> [1] TRUE

# Example 2: (i32) -> i32 - Unary integer operations
host_square <- function(x) as.integer(x * x)
host_double <- function(x) as.integer(x * 2)

square_func <- wasmer_function_new_i32_to_i32(runtime, host_square)
double_func <- wasmer_function_new_i32_to_i32(runtime, host_double)

# Example 3: (f64, f64) -> f64 - Binary float operations  
host_avg <- function(x, y) (x + y) / 2
host_max <- function(x, y) max(x, y)

avg_func <- wasmer_function_new_f64_f64_to_f64(runtime, host_avg)
max_func <- wasmer_function_new_f64_f64_to_f64(runtime, host_max)

# Example 4: (f64) -> f64 - Unary float operations
host_sqrt <- function(x) sqrt(x)
host_abs <- function(x) abs(x)

sqrt_func <- wasmer_function_new_f64_to_f64(runtime, host_sqrt)
abs_func <- wasmer_function_new_f64_to_f64(runtime, host_abs)

# Example 5: (i32) -> void - Side effects (logging/printing)
logged_values <- c()
host_log <- function(x) {
  logged_values <<- c(logged_values, x)
  invisible(NULL)
}

log_func <- wasmer_function_new_i32_to_void(runtime, host_log)

# Example 6: () -> i32 - Generators/timestamps
counter <- 0
host_next_id <- function() {
  counter <<- counter + 1
  as.integer(counter)
}

next_id_func <- wasmer_function_new_void_to_i32(runtime, host_next_id)

# Now create simple WASM modules that actually CALL these host functions!
## Example: Call Host Function from WASM Table


# Reuse the same runtime used above
# Host function: next_id generator
counter <- 0
host_next_id <- function() {
  counter <<- counter + 1
  as.integer(counter)
}
next_id_func <- wasmer_function_new_void_to_i32(runtime, host_next_id)

# Create WASM table and set host function

# Create WASM table with 4 slots and set host functions

# Create WASM table with 5 slots and set host functions

# Create WASM table with 9 slots and set host functions
table_ptr <- wasmer_table_new_ext(runtime, 9L, 9L)
wasmer_table_set_ext(runtime, table_ptr, 0L, next_id_func)   # slot 0: next_id
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 1L, add_func)       # slot 1: add
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 2L, square_func)    # slot 2: square
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 3L, mul_func)       # slot 3: multiply
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 4L, double_func)    # slot 4: double
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 5L, avg_func)       # slot 5: avg (f64, f64) -> f64
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 6L, max_func)       # slot 6: max (f64, f64) -> f64
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 7L, sqrt_func)      # slot 7: sqrt (f64) -> f64
#> [1] TRUE
wasmer_table_set_ext(runtime, table_ptr, 8L, abs_func)       # slot 8: abs (f64) -> f64
#> [1] TRUE

# WASM module that imports and uses a table
wat_table_call <- '
(module
  ;; Types for each host function signature
  (type $gen (func (result i32)))
  (type $binop (func (param i32 i32) (result i32)))
  (type $unop (func (param i32) (result i32)))
  (type $binopf64 (func (param f64 f64) (result f64)))
  (type $unopf64 (func (param f64) (result f64)))

  ;; Import a table with 9 funcref slots
  (import "env" "host_table" (table 9 funcref))

  ;; Call next_id (slot 0)
  (func $call_next_id (export "call_next_id") (result i32)
    (call_indirect (type $gen) (i32.const 0))
  )
  ;; Call add (slot 1)
  (func $call_add (export "call_add") (param $x i32) (param $y i32) (result i32)
    (call_indirect (type $binop) (local.get $x) (local.get $y) (i32.const 1))
  )
  ;; Call square (slot 2)
  (func $call_square (export "call_square") (param $x i32) (result i32)
    (call_indirect (type $unop) (local.get $x) (i32.const 2))
  )
  ;; Call multiply (slot 3)
  (func $call_multiply (export "call_multiply") (param $x i32) (param $y i32) (result i32)
    (call_indirect (type $binop) (local.get $x) (local.get $y) (i32.const 3))
  )
  ;; Call double (slot 4)
  (func $call_double (export "call_double") (param $x i32) (result i32)
    (call_indirect (type $unop) (local.get $x) (i32.const 4))
  )
  ;; Call avg (slot 5)
  (func $call_avg (export "call_avg") (param $x f64) (param $y f64) (result f64)
    (call_indirect (type $binopf64) (local.get $x) (local.get $y) (i32.const 5))
  )
  ;; Call max (slot 6)
  (func $call_max (export "call_max") (param $x f64) (param $y f64) (result f64)
    (call_indirect (type $binopf64) (local.get $x) (local.get $y) (i32.const 6))
  )
  ;; Call sqrt (slot 7)
  (func $call_sqrt (export "call_sqrt") (param $x f64) (result f64)
    (call_indirect (type $unopf64) (local.get $x) (i32.const 7))
  )
  ;; Call abs (slot 8)
  (func $call_abs (export "call_abs") (param $x f64) (result f64)
    (call_indirect (type $unopf64) (local.get $x) (i32.const 8))
  )
)
'

# Compile the module
wasmer_compile_wat_ext(runtime, wat_table_call, "table_module")
#> [1] "Module 'table_module' compiled successfully"

# Instantiate the module, passing the table as an import
wasmer_instantiate_with_table_ext(runtime, "table_module", "table_instance", table_ptr)
#> [1] "Instance 'table_instance' created successfully with table import"

# Call the WASM function, which calls the host function via the table

# Call each WASM function, which calls the corresponding host function via the table
result_next_id <- wasmer_call_function_ext(runtime, "table_instance", "call_next_id", list())
print(result_next_id)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 1

result_add <- wasmer_call_function_ext(runtime, "table_instance", "call_add", list(10L, 32L))
print(result_add)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 42

result_square <- wasmer_call_function_ext(runtime, "table_instance", "call_square", list(7L))
print(result_square)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 49

result_multiply <- wasmer_call_function_ext(runtime, "table_instance", "call_multiply", list(6L, 7L))
print(result_multiply)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 42

result_double <- wasmer_call_function_ext(runtime, "table_instance", "call_double", list(21L))
print(result_double)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 42

result_avg <- wasmer_call_function_ext(runtime, "table_instance", "call_avg", list(1.5, 2.5))
print(result_avg)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 2

result_max <- wasmer_call_function_ext(runtime, "table_instance", "call_max", list(1.5, 2.5))
print(result_max)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 2.5

result_sqrt <- wasmer_call_function_ext(runtime, "table_instance", "call_sqrt", list(9.0))
print(result_sqrt)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 3

result_abs <- wasmer_call_function_ext(runtime, "table_instance", "call_abs", list(-42.0))
print(result_abs)
#> $success
#> [1] TRUE
#> 
#> $values
#> [1] 42
```

### Available Typed Host Function Signatures

The package provides the following typed host function creators:

- `wasmer_function_new_i32_to_i32(runtime, rfun)` - (i32) -\> i32
- `wasmer_function_new_i32_i32_to_i32(runtime, rfun)` - (i32, i32) -\>
  i32  
- `wasmer_function_new_f64_f64_to_f64(runtime, rfun)` - (f64, f64) -\>
  f64
- `wasmer_function_new_f64_to_f64(runtime, rfun)` - (f64) -\> f64
- `wasmer_function_new_i32_to_void(runtime, rfun)` - (i32) -\> void (for
  logging)
- `wasmer_function_new_void_to_i32(runtime, rfun)` - () -\> i32 (for
  generators)

## LLM Usage Disclosure

Code and documentation in this project have been generated with the
assistance of the github Copilot LLM tools. While we have reviewed and
edited the generated content, we acknowledge that LLM tools were used in
the creation process and accordingly (since these models are trained on
GPL code and other commons + proprietary software license is fake
anyway) the code is released under GPL-3. So if you use this code in any
way, you must comply with the GPL-3 license.
