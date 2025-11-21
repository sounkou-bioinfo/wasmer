# Get exported WASM Table

Get a pointer to an exported table from a WASM instance by name.

## Usage

``` r
wasmer_get_exported_table_ext(ptr, instance_name, table_export_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

- table_export_name:

  Name of the exported table.

## Value

External pointer to Table, or NULL if not found

## Details

Get a pointer to an exported table from a WASM instance by name

## See also

[`wasmer_table_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_new_ext.md),
[`wasmer_table_set_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_set_ext.md),
[`wasmer_table_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_grow_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md)

Other table operations:
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_table_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_grow_ext.md),
[`wasmer_table_new_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_new_ext.md),
[`wasmer_table_set_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_set_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_get_exported_table_ext(ptr, "inst1", "table1")
} # }
```
