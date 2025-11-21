# Instantiate WASM module with table import

Instantiate a compiled WASM module in the runtime, with a custom table
import.

## Usage

``` r
wasmer_instantiate_with_table_ext(ptr, module_name, instance_name, table_ptr)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- module_name:

  Name of the module to instantiate.

- instance_name:

  Name to register the instance under.

- table_ptr:

  External pointer to Table to import as "env.host_table".

## Value

Status message

## Details

Instantiate a compiled module in the runtime, with a custom table import

## See also

[`wasmer_instantiate_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_ext.md),
[`wasmer_instantiate_with_math_imports_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_math_imports_ext.md),
[`wasmer_wasi_state_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wasi_state_new_ext.md)

Other module instantiation:
[`wasmer_instantiate_with_math_imports_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_instantiate_with_math_imports_ext.md),
[`wasmer_wasi_state_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wasi_state_new_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_instantiate_with_table_ext(ptr, "mod1", "inst1", table_ptr)
} # }
```
