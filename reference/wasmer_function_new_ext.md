# Create dynamic R host function

Create a Wasmer host function from an R function with dynamic signature.

## Usage

``` r
wasmer_function_new_ext(ptr, rfun, arg_types, ret_types, `_name`)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- rfun:

  R function object.

- arg_types:

  Character vector of argument types (e.g. c("i32", "f64")).

- ret_types:

  Character vector of return types (e.g. c("i32")).

- \_name:

  Character string for registry name.

## Value

External pointer to Function

## Details

Create a Wasmer host function from an R function with dynamic signature

## See also

[`wasmer_register_r_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_register_r_function_ext.md),
[`wasmer_function_new_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_i32.md),
[`wasmer_function_new_i32_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_i32_to_i32.md),
[`wasmer_function_new_f64_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_f64_to_f64.md),
[`wasmer_function_new_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_to_f64.md),
[`wasmer_function_new_i32_to_void()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_void.md),
[`wasmer_function_new_void_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_void_to_i32.md)

Other host function registration:
[`wasmer_function_new_f64_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_f64_to_f64.md),
[`wasmer_function_new_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_to_f64.md),
[`wasmer_function_new_i32_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_i32_to_i32.md),
[`wasmer_function_new_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_i32.md),
[`wasmer_function_new_i32_to_void()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_void.md),
[`wasmer_function_new_void_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_void_to_i32.md),
[`wasmer_register_r_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_register_r_function_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_function_new_ext(ptr, function(x) x, c("i32"), c("i32"), "myfun")
} # }
```
