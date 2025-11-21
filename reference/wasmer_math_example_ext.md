# Math operations example

Example WASM module for math operations.

## Usage

``` r
wasmer_math_example_ext(ptr, a, b)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- a:

  First integer.

- b:

  Second integer.

## Value

List with results of add and multiply

## Details

Math operations example

## See also

[`wasmer_call_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_ext.md),
[`wasmer_call_function_safe_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_safe_ext.md),
[`wasmer_host_function_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_host_function_example_ext.md),
[`wasmer_hello_world_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_hello_world_example_ext.md)

Other function calling:
[`wasmer_call_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_ext.md),
[`wasmer_call_function_safe_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_safe_ext.md),
[`wasmer_hello_world_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_hello_world_example_ext.md),
[`wasmer_host_function_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_host_function_example_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_math_example_ext(ptr, 2, 3)
} # }
```
