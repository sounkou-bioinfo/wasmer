# Call WASM function (type safe)

Call an exported WASM function with type safety and conversion.

## Usage

``` r
wasmer_call_function_safe_ext(ptr, instance_name, function_name, args)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  String name of the instance.

- function_name:

  String name of the function to call.

- args:

  List of arguments with proper type conversion.

## Value

List with success flag and result or error

## Details

Advanced function calling with type safety

## See also

[`wasmer_call_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_ext.md),
[`wasmer_host_function_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_host_function_example_ext.md),
[`wasmer_math_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_math_example_ext.md),
[`wasmer_hello_world_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_hello_world_example_ext.md)

Other function calling:
[`wasmer_call_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_call_function_ext.md),
[`wasmer_hello_world_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_hello_world_example_ext.md),
[`wasmer_host_function_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_host_function_example_ext.md),
[`wasmer_math_example_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_math_example_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_call_function_safe_ext(ptr, "inst1", "add", list(1, 2))
} # }
```
