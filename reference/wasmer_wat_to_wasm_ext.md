# Convert WAT to WASM

Convert WebAssembly Text (WAT) to WASM binary and return as R raw
vector.

## Usage

``` r
wasmer_wat_to_wasm_ext(wat_code)
```

## Arguments

- wat_code:

  WAT code as a string.

## Value

WASM binary as R raw vector, or error string if conversion fails

## Details

Convert WAT (WebAssembly Text) to WASM binary and return as R raw vector

## See also

[`wasmer_compile_wat_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wat_ext.md),
[`wasmer_compile_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wasm_ext.md)

Other module compilation:
[`wasmer_compile_wasm_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wasm_ext.md),
[`wasmer_compile_wat_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_compile_wat_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_wat_to_wasm_ext(wat_code)
} # }
```
