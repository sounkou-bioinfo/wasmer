# Grow WASM memory

Grow WASM memory by a number of pages.

## Usage

``` r
wasmer_memory_grow_ext(ptr, instance_name, memory_name, pages)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

- memory_name:

  Name of the exported memory.

- pages:

  Number of pages to grow.

## Value

TRUE if successful

## Details

Grow WASM memory by a number of pages

## See also

[`wasmer_memory_size_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_size_ext.md),
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_write_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_write_ext.md),
[`wasmer_memory_read_string_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_string_ext.md)

Other memory operations:
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_read_string_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_string_ext.md),
[`wasmer_memory_size_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_size_ext.md),
[`wasmer_memory_write_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_write_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_memory_grow_ext(ptr, "inst1", "memory", 1)
} # }
```
