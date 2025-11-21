# Create WASI/WASIX state

Create a WASI or WASIX state for the runtime.

## Usage

``` r
wasmer_wasi_state_new_ext(ptr, module_name, env_type)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- module_name:

  Name of the module (for WASI/WASIX args).

- env_type:

  Environment type: "wasi" (default) or "wasix".

## Value

TRUE if successful, FALSE otherwise

## Details

Create a WASI or WASIX state for the runtime

## See also

[`wasmer_instantiate_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_ext.md),
[`wasmer_instantiate_with_math_imports_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_math_imports_ext.md),
[`wasmer_instantiate_with_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_table_ext.md)

Other module instantiation:
[`wasmer_instantiate_with_math_imports_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_math_imports_ext.md),
[`wasmer_instantiate_with_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_table_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_wasi_state_new_ext(ptr, "mod1", "wasi")
} # }
```
