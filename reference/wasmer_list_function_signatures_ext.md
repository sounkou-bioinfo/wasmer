# List WASM function signatures

List exported function signatures (name, input types, output types) for
a WASM instance.

## Usage

``` r
wasmer_list_function_signatures_ext(ptr, instance_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

## Value

Data frame with columns: name, params, results

## Details

List exported function signatures (name, input types, output types) for
a WASM instance

## See also

[`wasmer_list_exports_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_list_exports_ext.md)

Other exports and signatures:
[`wasmer_list_exports_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_list_exports_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_list_function_signatures_ext(ptr, "inst1")
} # }
```
