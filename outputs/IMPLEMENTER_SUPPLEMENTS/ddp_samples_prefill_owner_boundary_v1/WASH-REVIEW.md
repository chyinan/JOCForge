# Wash review: DDP source-map samples prefill boundary

The bounded DDP call chain binds a metadata preflight that reads `filename` from the source map at context `+0xa0` and writes selected metadata to a separate output map. Neither the preflight nor the context initializer writes the source `samples` member. The complete-map branch therefore requires a preexisting `samples` value, but the source map writer is outside this bounded chain. The callback thunk has no direct code callers in the reviewed image; its one data-address use does not establish dispatch semantics.

The minimum runtime identity token is the DDP context pointer read from callback receiver `+0xb8` and passed unchanged to the context initializer. To identify the source of `samples`, bind that same context identity and generation through the first write of the `+0xa0` map member and its complete-map check. Do not substitute MediaInfo or any other owner based on a matching key/value.

This is a source metadata ownership boundary only; it supplies no audio algorithm or parameter.