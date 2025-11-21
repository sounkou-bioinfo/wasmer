# Instantiate WASM module with math imports

Instantiate a WASM module with host functions for mathematical
operations.

## Usage

``` r
wasmer_instantiate_with_math_imports_ext(ptr, module_name, instance_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- module_name:

  String name of the module to instantiate.

- instance_name:

  String name to identify this instance.

## Value

Status message

## Details

Create an instance with host functions for mathematical operations

## See also

[`wasmer_instantiate_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_ext.md),
[`wasmer_instantiate_with_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_table_ext.md),
[`wasmer_wasi_state_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wasi_state_new_ext.md)

Other module instantiation:
[`wasmer_instantiate_with_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_table_ext.md),
[`wasmer_wasi_state_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wasi_state_new_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_instantiate_with_math_imports_ext(ptr, "mod1", "inst1")
} # }
```
