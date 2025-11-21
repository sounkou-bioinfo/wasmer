# Set WASM Table entry

Set a function reference in a WASM Table.

## Usage

``` r
wasmer_table_set_ext(ptr, table_ptr, index, func_ptr)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- table_ptr:

  External pointer to Table.

- index:

  Index to set.

- func_ptr:

  External pointer to Function.

## Value

TRUE if successful

## Details

Set a function reference in a WASM Table

## See also

[`wasmer_table_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_new_ext.md),
[`wasmer_table_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_grow_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_get_exported_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_get_exported_table_ext.md)

Other table operations:
[`wasmer_get_exported_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_get_exported_table_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_table_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_grow_ext.md),
[`wasmer_table_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_new_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_table_set_ext(ptr, table_ptr, 0, func_ptr)
} # }
```
