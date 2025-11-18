# Example: Fibonacci Sequence in WebAssembly

library(wasmer)

# Initialize the Wasmer runtime
wasmer_init()

# Define a Fibonacci function in WebAssembly Text Format
fibonacci_wat <- '
(module
  (func $fibonacci (export "fibonacci") (param $n i32) (result i32)
    (local $a i32)
    (local $b i32)
    (local $i i32)

    ;; Handle base cases
    (if (i32.le_s (local.get $n) (i32.const 1))
      (then (return (local.get $n)))
    )

    ;; Initialize variables
    (local.set $a (i32.const 0))
    (local.set $b (i32.const 1))
    (local.set $i (i32.const 2))

    ;; Fibonacci loop
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

# Compile the WebAssembly module
compile_result <- wasmer_compile_wat(fibonacci_wat, "fibonacci_module")
cat("Compilation result:", compile_result, "\n")

# Create an instance of the module
instance_result <- wasmer_instantiate("fibonacci_module", "fib_instance")
cat("Instantiation result:", instance_result, "\n")

# List available exports
exports <- wasmer_list_exports("fib_instance")
cat("Available exports:\n")
print(exports)

# Calculate fibonacci numbers
cat("\nFibonacci sequence:\n")
for (i in 0:15) {
    result <- wasmer_call_function_safe("fib_instance", "fibonacci", list(i))
    if (result$success) {
        cat(sprintf("fib(%d) = %d\n", i, result$values[[1]]))
    } else {
        cat(sprintf("Error calculating fib(%d): %s\n", i, result$error))
    }
}
