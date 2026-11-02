# DEE core frame-provider boundary

The selected caller requests an output span and a frame descriptor from its frame-provider virtual slot. A native backend selector chooses between a core `EmpireEnc` provider and an `AtmosProcessor` provider, but the selector value for the authorized job is not captured. The core provider's slot wrapper forwards an inner state and request descriptor into a processor that accepts only 1536 samples per normal request or zero for flush.

The normal request path prepares an encoder context, stages the 1536-sample block, calls `ddp_joc_pro_encode_process` into a scratch buffer, and returns an encoded span using the actual byte count. This is an encoding provider, not a pass-through of prebuilt frame bytes. The bounded trace does not bind sample-rate, `acmod`, `lfeon`, or wire channel order to typed request fields; the sample-count and channel-order semantics of the request remain partially opaque.

The provider consumes audio input and could therefore sit before a PCM-affecting difference, but this trace did not observe a selected per-run backend, sample mapping, or output effect. It neither confirms nor explains the AU-aligned PCM mismatch. The shortest next static edge is the request/config input to the encoder-open helper, where format semantics may be found. No source/decompilation or private material is included.
