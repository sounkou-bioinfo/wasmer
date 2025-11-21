# Get WASM memory size

Get the size of exported memory (in bytes and pages).

## Usage

``` r
wasmer_memory_size_ext(ptr, instance_name, memory_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

- memory_name:

  Name of the exported memory (default "memory").

## Value

List with size_bytes and size_pages

## Details

Get the size of exported memory (in bytes and pages)

## See also

[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_write_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_write_ext.md),
[`wasmer_memory_read_string_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_string_ext.md),
[`wasmer_memory_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_grow_ext.md)

Other memory operations:
[`wasmer_memory_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_grow_ext.md),
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_read_string_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_string_ext.md),
[`wasmer_memory_write_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_write_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_memory_size_ext(ptr, "inst1", "memory")
} # }
```
