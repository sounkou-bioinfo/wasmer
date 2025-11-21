# Register R host function

Register an R function for use as a host function in WASM (per-runtime).

## Usage

``` r
wasmer_register_r_function_ext(ptr, name, fun)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- fun:

  R function object.

- \_name:

  Name to register the function under.

## Value

TRUE if successful

## Details

Register an R function for use as a host function in WASM (per-runtime)

## See also

[`wasmer_function_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_ext.md),
[`wasmer_function_new_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_i32.md),
[`wasmer_function_new_i32_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_i32_to_i32.md),
[`wasmer_function_new_f64_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_f64_to_f64.md),
[`wasmer_function_new_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_to_f64.md),
[`wasmer_function_new_i32_to_void()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_void.md),
[`wasmer_function_new_void_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_void_to_i32.md)

Other host function registration:
[`wasmer_function_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_ext.md),
[`wasmer_function_new_f64_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_f64_to_f64.md),
[`wasmer_function_new_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_to_f64.md),
[`wasmer_function_new_i32_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_i32_to_i32.md),
[`wasmer_function_new_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_i32.md),
[`wasmer_function_new_i32_to_void()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_void.md),
[`wasmer_function_new_void_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_void_to_i32.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_register_r_function_ext(ptr, "myfun", function(x) x)
} # }
```
