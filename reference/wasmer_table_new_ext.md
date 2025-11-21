# Create WASM Table

Create a new WASM Table.

## Usage

``` r
wasmer_table_new_ext(ptr, min, max)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- min:

  Minimum size.

- max:

  Maximum size (optional).

## Value

External pointer to Table

## Details

Create a new WASM Table

## See also

[`wasmer_table_set_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_set_ext.md),
[`wasmer_table_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_grow_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_get_exported_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_get_exported_table_ext.md)

Other table operations:
[`wasmer_get_exported_table_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_get_exported_table_ext.md),
[`wasmer_table_get_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_get_ext.md),
[`wasmer_table_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_grow_ext.md),
[`wasmer_table_set_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_table_set_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_table_new_ext(ptr, 1, 10)
} # }
```
