# Sample 6 host-to-core frame ABI boundary

Recipient: implementation evidence supplement  
Disposition: core frame adapter is known locally; the selected ADM reader's transfer into it is not bound.

## Local core API contract

In the hash-bound DDP encoder image, the DdpEncoderApp and Comp process vtable slots resolve to `FUN_140188850`, `FUN_140214ef0`, and `FUN_140214e80`. The two Comp wrappers call `FUN_140215070`, which calls `FUN_140213470`. At that local API boundary, frame `param3+0x20` is a U64 count, `param3+0x08` is a U64 channel pointer-list address, and the count is narrowed to U32 in a local descriptor. This establishes the local frame-view-to-descriptor transfer only.

The same-image scan found no code caller of the Comp process vtable slots; observed refs are vtable/data, `.pdata`, or exception metadata. External `this`, frame-view construction, and caller ownership are therefore not identified by this package.

## Host and Atmos boundary

The host image has an `atmos_mezz` reader `Run` vtable slot (`FUN_1400F09D0`) and metadata slot (`FUN_1400F0130`). Its checked reader path dispatches through a reader interface and writes host metadata, but no checked code call reaches the core Comp process methods. The host dynamically loads the `audio_filter` export; the Atmos DLL's own process wrapper validates its separate frame packet at `param3[2]` for 1536 samples or flush. There is no typed pointer, count, object identity, or generation join from that Atmos packet to the core frame view.

## Remaining edge

The selected `atmos_mezz` reader count → external filter instance/process callback → core frame view count and pointer list remains `UNBOUND`. Equal numeric counts and shared 1536-sample checks do not establish the transfer. A 36/38 ADM track count therefore cannot yet be identified with core frame `param3+0x20`.

This is Ghidra 12.2 static evidence only. No DEE run, runtime capture, PCM/payload/key read, or audio algorithm change occurred.
