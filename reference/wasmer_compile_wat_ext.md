# Compile WAT module

Compile a WebAssembly Text (WAT) module and add it to the runtime.

## Usage

``` r
wasmer_compile_wat_ext(ptr, wat_code, module_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- wat_code:

  WAT code as a string.

- module_name:

  Name to register the module under.

## Value

Status message

## Details

Compile a WAT (WebAssembly Text) module and add it to the runtime

## See also

[`wasmer_compile_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wasm_ext.md),
[`wasmer_wat_to_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wat_to_wasm_ext.md)

Other module compilation:
[`wasmer_compile_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wasm_ext.md),
[`wasmer_wat_to_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_wat_to_wasm_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_compile_wat_ext(ptr, wat_code, "mod1")
} # }
```
