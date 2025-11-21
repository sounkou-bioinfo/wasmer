# Call WASM function

Call an exported function from a WASM instance.

## Usage

``` r
wasmer_call_function_ext(ptr, instance_name, function_name, args)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

- function_name:

  Name of the function to call.

- args:

  Arguments as R list.

## Value

List with success flag and result or error

## Details

Call an exported function from a WASM instance

## See also

[`wasmer_call_function_safe_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_safe_ext.md),
[`wasmer_host_function_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_host_function_example_ext.md),
[`wasmer_math_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_math_example_ext.md),
[`wasmer_hello_world_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_hello_world_example_ext.md)

Other function calling:
[`wasmer_call_function_safe_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_safe_ext.md),
[`wasmer_hello_world_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_hello_world_example_ext.md),
[`wasmer_host_function_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_host_function_example_ext.md),
[`wasmer_math_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_math_example_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_call_function_ext(ptr, "inst1", "add", list(1, 2))
} # }
```
