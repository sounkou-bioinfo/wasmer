# Wasmer R Package Examples

This directory contains example WebAssembly modules and R scripts demonstrating 
how to use the wasmer package.

## Basic Usage

```r
library(wasmer)

# Initialize the runtime
wasmer_init()

# Run a simple hello world example
result <- wasmer_hello_world_example()
print(result)

# Run a math example
math_result <- wasmer_math_example(5L, 3L)
print(math_result)
```

## Advanced Usage

```r
library(wasmer)

# Initialize runtime
wasmer_init()

# Define WebAssembly text format code
wat_code <- '
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

# Compile the module
compile_result <- wasmer_compile_wat(wat_code, "fibonacci_module")
print(compile_result)

# Instantiate the module
instance_result <- wasmer_instantiate("fibonacci_module", "fib_instance")
print(instance_result)

# List available exports
exports <- wasmer_list_exports("fib_instance")
print(exports)

# Call the fibonacci function
fib_10 <- wasmer_call_function_safe("fib_instance", "fibonacci", list(10L))
print(fib_10)
```

## Host Function Integration

```r
library(wasmer)

wasmer_init()

# Example that uses host functions
result <- wasmer_host_function_example()
print(result)
```
