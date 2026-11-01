# Wash review: R138 ADM time-normalizer candidate

This Ghidra-only package links the hash-bound R138 ADM schema to a candidate host normalizer path, but it does not establish that the candidate internal run method was selected for the actual R138 invocation.

The schema includes timecode_frame_rate, offset, and ffoa. R138 uses ADM input, the encode_to_atmos_ddp route, embedded_timecode, numeric start/end tokens, not_indicated frame rate, and auto offset/FFOA. Static code shows an ADM factory calling an internal input-object constructor; its run method and a separate WAV run method both call the shared source-time normalizer. The normalizer reads offset/FFOA, applies a rate-aware helper for explicit timecode, and writes normalized values into a host property map.

The DDP setter independently copies record slots 10/11 to binary64 seconds later read as offset/FFOA. The normalized host map is not statically joined to the setter record. R138 internal-run selection, map owner/generation, helper argument meaning, resolved FPS, setter callsite, and end-to-end behavior remain UNBOUND/NOT_CAPTURED/NOT_RUN.

The internal class name must not be substituted for the user-facing input type: the bound R138 profile remains ADM. No native job or audio payload was used in this static package.
