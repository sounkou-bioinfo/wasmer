# Grow WASM Table

Grow a WASM Table by a number of elements.

## Usage

``` r
wasmer_table_grow_ext(ptr, table_ptr, delta, func_ptr)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- table_ptr:

  External pointer to Table.

- delta:

  Number of elements to grow.

- func_ptr:

  External pointer to Function to fill new slots.

## Value

Previous size

## Details

Grow a WASM Table

## See also

[`wasmer_table_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_new_ext.md),
[`wasmer_table_set_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_set_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_get_exported_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_get_exported_table_ext.md)

Other table operations:
[`wasmer_get_exported_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_get_exported_table_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_table_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_new_ext.md),
[`wasmer_table_set_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_set_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_table_grow_ext(ptr, table_ptr, 1, func_ptr)
} # }
```
