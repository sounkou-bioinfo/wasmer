# Create a new Wasmer runtime

Create a new Wasmer runtime for executing WebAssembly modules.

## Usage

``` r
wasmer_runtime_new()
```

## Value

External pointer to WasmerRuntime

## Details

Create a new Wasmer runtime

## See also

[`wasmer_runtime_new_with_compiler_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new_with_compiler_ext.md),
[`wasmer_runtime_release_ressources()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_release_ressources.md)

Other runtime management:
[`wasmer_runtime_new_with_compiler_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new_with_compiler_ext.md),
[`wasmer_runtime_release_ressources()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_release_ressources.md)

## Examples

``` r
if (FALSE) { # \dontrun{
ptr <- wasmer_runtime_new()
} # }
```
