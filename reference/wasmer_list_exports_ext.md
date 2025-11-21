# List WASM exports

List all exports from a WASM instance.

## Usage

``` r
wasmer_list_exports_ext(ptr, instance_name)
```

## Arguments

- ptr:

  External pointer to WasmerRuntime.

- instance_name:

  Name of the instance.

## Value

List with success flag and exports or error

## Details

List all exports from a WASM instance

## See also

[`wasmer_list_function_signatures_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_list_function_signatures_ext.md)

Other exports and signatures:
[`wasmer_list_function_signatures_ext()`](https://sounkou-bioinfo.github.io/wasmer/reference/wasmer_list_function_signatures_ext.md)

## Examples

``` r
if (FALSE) { # \dontrun{
wasmer_list_exports_ext(ptr, "inst1")
} # }
```
