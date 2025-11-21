# Compile WASM binary

Compile a WebAssembly binary and add it to the runtime.

## Usage

``` r
wasmer_compile_wasm_ext(ptr, wasm_bytes, module_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- wasm_bytes:

  WASM binary as R raw vector.

- module_name:

  Name to register the module under.

## Value

Status message

## Details

Compile a WASM binary and add it to the runtime

## See also

[`wasmer_compile_wat_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wat_ext.md),
[`wasmer_wat_to_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wat_to_wasm_ext.md)

Other module compilation:
[`wasmer_compile_wat_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wat_ext.md),
[`wasmer_wat_to_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wat_to_wasm_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_compile_wasm_ext(ptr, wasm_bytes, "mod1")
} # }
```
