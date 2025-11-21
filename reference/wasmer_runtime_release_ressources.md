# Release Wasmer runtime resources

Explicitly shutdown the runtime, free resources, and clear the R
external pointer.

## Usage

``` r
wasmer_runtime_release_ressources(ptr)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

## Value

NULL (invisible)

## Details

Release resources held by the Wasmer runtime

## See also

[`wasmer_runtime_new()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new.md),
[`wasmer_runtime_new_with_compiler_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new_with_compiler_ext.md)

Other runtime management:
[`wasmer_runtime_new()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new.md),
[`wasmer_runtime_new_with_compiler_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new_with_compiler_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_runtime_release_ressources(ptr)
} # }
```
