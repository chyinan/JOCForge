# ID14 model parser and native BSI source clarification

This internal supplement distinguishes one frozen model/helper path from the selected native outgoing E-AC-3/JOC path. It supersedes only the attribution of the selected BSI complexity scalar; it does not remove or deny the model helper algorithm.

## Model/helper fact

The hash-bound frozen helper `codec_core.rs` contains `AncillaryPayload::validated_at_block`. It parses a complete EMDF wrapper, requires exact byte consumption, applies the selected profile checks, parses ID14/OAMD semantically, and computes `object_count + 1` for that helper's additional-BSI result. `MODEL-PARSER-VECTORS.json` preserves 13 current model-helper input/observation cases. The captured debug and release outputs agree. This is model-helper execution, not a native parser test. The OpenJOC dependency sources were hash-pinned for this run, but their historical frozen identity is not established beyond the helper's dependency references.

## Selected native BSI fact

The selected native BSI producer reads the validated configuration-count payload, carries it into the codec owner, and emits the designated complexity-index byte before the outgoing EMDF serializer runs. The already approved `F13C-BSI-COUNT-PRODUCER` and `F13C-SELECTED-ID14-CONSUMPTION-R2` packages provide direct native cases for configured counts 1 and 16 while ID14 body, queue, serializer and JOC-body storage are absent. R2's selected-scope overlay replaces the old claim that this BSI count must be derived by parsing serialized ID14.

For the selected helper vector, `object_count + 1` is numerically 16, matching the selected configured count 16. That numerical equality does not identify the producer. Implement the selected native BSI source from validated configuration count; keep the helper's parser/count formula as model-helper behavior if that validation helper is implemented. Do not parse the outgoing ID14 body to derive the native BSI count.

## Parser boundary

The outgoing selected path produces and queues ID14 as an opaque payload descriptor and serializes it as part of EMDF. Generic container/descriptor collection is not an ID14 semantic parser. No native ID14 semantic-reader ABI was found on the traced selected path, and native malformed/full-container semantic parser cases were NOT_RUN. This is a scoped observation, not a claim that no parser exists anywhere in the DLL.

The model vectors include an ID14-only container that is rejected by the selected OAMD+JOC profile because OAMD is absent. A complete valid selected OAMD+ID14 wrapper success case was NOT_RUN. Whole F13C remains NOT_READY.

## Provenance and limits

The approved complexity spec identity is bound in `PROVENANCE.json`. Native source identity is bound only by the module SHA-256; no DLL, disassembly, raw function addresses, source paths, key, authentication values or PCM are included. Existing BSI producer evidence is referenced by its washed manifest/check hashes. This supplement is for internal implementation use.
