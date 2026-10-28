# AU token equality: bounded validation path and stop result

A bounded selected path observes a threshold selector choosing a non-null pointer token and passing it unchanged to a direct helper. The helper returns either a position or a length sentinel; the caller compares that result with the 4096-byte input length and treats a non-length result as validation failure.

This binds slot-to-validation-helper token equality only. The bounded path contains no transport enqueue or AU-descriptor operation. The selected transport AU identity is NOT_RUN/UNBOUND; source generation is also UNBOUND. Stop this token branch rather than relabeling a validation/search token as an AU token.

The only useful future observation is the first caller-side transport publication boundary after the processing loop returns, if that path is resumed. No payload bytes, native code/addresses, key or authentication material are included.
