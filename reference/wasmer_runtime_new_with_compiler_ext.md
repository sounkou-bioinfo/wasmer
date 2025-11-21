# Create a new Wasmer runtime with a specific compiler

Create a new Wasmer runtime for executing WebAssembly modules using a
specified compiler backend.

## Usage

``` r
wasmer_runtime_new_with_compiler_ext(compiler_name)
```

## Arguments

- compiler_name:

  Name of the compiler ("cranelift", "singlepass").

## Value

External pointer to WasmerRuntime

## Details

Create a new Wasmer runtime with a specific compiler

## See also

[`wasmer_runtime_new()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new.md),
[`wasmer_runtime_release_ressources()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_release_ressources.md)

Other runtime management:
[`wasmer_runtime_new()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_new.md),
[`wasmer_runtime_release_ressources()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_runtime_release_ressources.md)

## Examples

``` r
if (FALSE) { # \dontrun{
ptr <- wasmer_runtime_new_with_compiler_ext("cranelift")
} # }
```
