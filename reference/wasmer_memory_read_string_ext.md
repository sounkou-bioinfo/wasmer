# Read WASM memory as string

Read UTF-8 string from WASM memory.

## Usage

``` r
wasmer_memory_read_string_ext(ptr, instance_name, memory_name, offset, length)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

- memory_name:

  Name of the exported memory.

- offset:

  Offset to start reading.

- length:

  Number of bytes to read.

## Value

String

## Details

Read UTF-8 string from WASM memory

## See also

[`wasmer_memory_size_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_size_ext.md),
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_write_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_write_ext.md),
[`wasmer_memory_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_grow_ext.md)

Other memory operations:
[`wasmer_memory_grow_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_grow_ext.md),
[`wasmer_memory_read_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_read_ext.md),
[`wasmer_memory_size_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_size_ext.md),
[`wasmer_memory_write_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_memory_write_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_memory_read_string_ext(ptr, "inst1", "memory", 0, 10)
} # }
```
