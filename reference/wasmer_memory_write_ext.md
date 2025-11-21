# Write WASM memory

Write bytes to WASM memory.

## Usage

``` r
wasmer_memory_write_ext(ptr, instance_name, memory_name, offset, bytes)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

- memory_name:

  Name of the exported memory.

- offset:

  Offset to start writing.

- bytes:

  Raw vector of bytes to write.

## Value

TRUE if successful

## Details

Write bytes to WASM memory

## See also

[`wasmer_memory_size_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_size_ext.md),
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_read_string_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_string_ext.md),
[`wasmer_memory_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_grow_ext.md)

Other memory operations:
[`wasmer_memory_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_grow_ext.md),
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_read_string_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_string_ext.md),
[`wasmer_memory_size_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_size_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_memory_write_ext(ptr, "inst1", "memory", 0, as.raw(c(1,2,3)))
} # }
```
