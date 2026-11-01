# R138 component receiver first-store boundary

## Correction and result

The earlier exact-displacement scan found no `+0x54` writer joined to the callback receiver. The latest typed trace finds why: the receiver's first write covering `+0x54` is a 16-byte copy beginning at `+0x50`, which an exact `+0x54` direct-operand scan cannot see. The receiver owner and first store are now bound. Its source value from the R138 job decoder remains `UNBOUND`.

## Typed chain

In the hash-bound Atmos image, the frame callback passes the frame record's embedded object at `+0xd8` to a wrapper. The wrapper allocates a 568-byte receiver and calls its initializer. The initializer copies 16 bytes from the source object's `+0x50` block to the new receiver's `+0x50` block; bytes 4–7 of that copy cover receiver `+0x54`.

The frame-record builder assigns frame-record `+0x12c` from process-state `+0x7e4`. The receiver pointer is retained at wrapper `+0x08` and reloaded for the component callback. This binds the receiver's first store and owner through the component callback.

The process-state `+0x7e4` field has not been traced back to the R138 job decoder. Therefore the R138-specific value and whether custom BSI zero applies remain `UNBOUND`.

## Evidence and limits

Incoming package `native-f06-r138-component-receiver-source-chain-addendum` manifest SHA-256: `5705697e89bd314346637742db1151ffc84062449e26a78163600d3522f1ac37`. Normal and `python -O` checkers passed. Target image SHA-256 is `3d66bcec36031fd48e6565d15f05fea656642377ca4f8c98cdce1cce8b7e95d2`; R138 job SHA-256 is `fa65143f0bd7de52b36367137e61d681baa4c17824aded10be26317e7d1018a4`. Input identity is the authorized “人是猫” ADM, SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`.

The source package discloses earlier out-of-scope scans and repeated vtable probes; they are excluded from this typed-chain claim. No new DEE run, U16 capture, media/payload/key read, or implementation contact occurred.

## Next edge

Find the first writer/configurer of process-state `+0x7e4` and type its source back to the R138 job decoder. Do not infer its value from the frame-record copy or the callback receiver identity.
