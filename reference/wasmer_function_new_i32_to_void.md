# Create host function (i32 -\> void)

Create a WASM host function that takes an i32 and returns nothing, using
an R function as the implementation.

## Usage

``` r
wasmer_function_new_i32_to_void(ptr, rfun)
```

## Details

Create a WASM host function with signature i32 -\> void

## See also

[`wasmer_register_r_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_register_r_function_ext.md),
[`wasmer_function_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_ext.md),
[`wasmer_function_new_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_i32.md),
[`wasmer_function_new_i32_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_i32_to_i32.md),
[`wasmer_function_new_f64_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_f64_to_f64.md),
[`wasmer_function_new_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_to_f64.md),
[`wasmer_function_new_void_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_void_to_i32.md)

Other host function registration:
[`wasmer_function_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_ext.md),
[`wasmer_function_new_f64_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_f64_to_f64.md),
[`wasmer_function_new_f64_to_f64()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_f64_to_f64.md),
[`wasmer_function_new_i32_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_i32_to_i32.md),
[`wasmer_function_new_i32_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_i32_to_i32.md),
[`wasmer_function_new_void_to_i32()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_function_new_void_to_i32.md),
[`wasmer_register_r_function_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_register_r_function_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_function_new_i32_to_void(ptr, function(x) cat(x))
} # }
```
