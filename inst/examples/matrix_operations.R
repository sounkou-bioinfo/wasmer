# Example: Matrix Operations in WebAssembly

library(wasmer)

# Initialize the runtime
wasmer_init()

# Define matrix operations in WebAssembly
matrix_wat <- '
(module
  ;; Memory for storing matrices (each i32 is 4 bytes)
  (memory (export "memory") 1)

  ;; Matrix addition: C = A + B (all 2x2 matrices)
  (func $matrix_add (export "matrix_add")
    (param $a_ptr i32) (param $b_ptr i32) (param $c_ptr i32)
    (local $i i32)

    ;; Add corresponding elements
    (loop $add_loop
      (i32.store
        (i32.add (local.get $c_ptr) (i32.mul (local.get $i) (i32.const 4)))
        (i32.add
          (i32.load (i32.add (local.get $a_ptr) (i32.mul (local.get $i) (i32.const 4))))
          (i32.load (i32.add (local.get $b_ptr) (i32.mul (local.get $i) (i32.const 4))))
        )
      )

      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br_if $add_loop (i32.lt_u (local.get $i) (i32.const 4)))
    )
  )

  ;; Get element at position (row, col) from 2x2 matrix
  (func $get_element (export "get_element")
    (param $matrix_ptr i32) (param $row i32) (param $col i32) (result i32)
    (i32.load
      (i32.add
        (local.get $matrix_ptr)
        (i32.mul
          (i32.add
            (i32.mul (local.get $row) (i32.const 2))
            (local.get $col)
          )
          (i32.const 4)
        )
      )
    )
  )

  ;; Set element at position (row, col) in 2x2 matrix
  (func $set_element (export "set_element")
    (param $matrix_ptr i32) (param $row i32) (param $col i32) (param $value i32)
    (i32.store
      (i32.add
        (local.get $matrix_ptr)
        (i32.mul
          (i32.add
            (i32.mul (local.get $row) (i32.const 2))
            (local.get $col)
          )
          (i32.const 4)
        )
      )
      (local.get $value)
    )
  )
)
'

# Compile and instantiate the module
compile_result <- wasmer_compile_wat(matrix_wat, "matrix_module")
cat("Compilation:", compile_result, "\n")

instance_result <- wasmer_instantiate("matrix_module", "matrix_instance")
cat("Instantiation:", instance_result, "\n")

# List available functions
exports <- wasmer_list_exports("matrix_instance")
cat("Available functions:\n")
print(exports)

# Example: Set up two 2x2 matrices and add them
cat("\nSetting up matrices...\n")

# Matrix A at memory location 0: [[1, 2], [3, 4]]
wasmer_call_function_safe("matrix_instance", "set_element", list(0L, 0L, 0L, 1L)) # A[0,0] = 1
wasmer_call_function_safe("matrix_instance", "set_element", list(0L, 0L, 1L, 2L)) # A[0,1] = 2
wasmer_call_function_safe("matrix_instance", "set_element", list(0L, 1L, 0L, 3L)) # A[1,0] = 3
wasmer_call_function_safe("matrix_instance", "set_element", list(0L, 1L, 1L, 4L)) # A[1,1] = 4

# Matrix B at memory location 16: [[5, 6], [7, 8]]
wasmer_call_function_safe("matrix_instance", "set_element", list(16L, 0L, 0L, 5L)) # B[0,0] = 5
wasmer_call_function_safe("matrix_instance", "set_element", list(16L, 0L, 1L, 6L)) # B[0,1] = 6
wasmer_call_function_safe("matrix_instance", "set_element", list(16L, 1L, 0L, 7L)) # B[1,0] = 7
wasmer_call_function_safe("matrix_instance", "set_element", list(16L, 1L, 1L, 8L)) # B[1,1] = 8

# Add matrices: C = A + B (result at memory location 32)
wasmer_call_function_safe("matrix_instance", "matrix_add", list(0L, 16L, 32L))

# Read result matrix C
cat("Matrix A:\n")
for (row in 0:1) {
    for (col in 0:1) {
        result <- wasmer_call_function_safe("matrix_instance", "get_element", list(0L, row, col))
        cat(result$values[[1]], " ")
    }
    cat("\n")
}

cat("\nMatrix B:\n")
for (row in 0:1) {
    for (col in 0:1) {
        result <- wasmer_call_function_safe("matrix_instance", "get_element", list(16L, row, col))
        cat(result$values[[1]], " ")
    }
    cat("\n")
}

cat("\nMatrix C = A + B:\n")
for (row in 0:1) {
    for (col in 0:1) {
        result <- wasmer_call_function_safe("matrix_instance", "get_element", list(32L, row, col))
        cat(result$values[[1]], " ")
    }
    cat("\n")
}
