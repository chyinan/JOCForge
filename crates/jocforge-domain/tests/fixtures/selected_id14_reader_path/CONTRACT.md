# Selected outgoing ID14 semantic-reader applicability

The selected native outgoing encode path does not execute an ID14 semantic reader in the recovered control-flow contract. The native ID14 body is produced, placed in a generic descriptor/queue entry, returned as that descriptor, reserved for output, and serialized as opaque bytes. The selected ingress dispatch semantically parses ID11 only; ID14 follows the generic queue-copy route. Generic EMDF container readers count/materialize declared payload bytes and apply framing rules, but do not dispatch ID14 semantic fields.

Therefore native ID14 malformed semantic-parser behavior is NOT_APPLICABLE to this recovered outgoing encode path. Do not block the selected writer/serializer on implementing a native ID14 semantic parser, and do not fabricate a native parser error, consumed-bit count or rollback behavior. The frozen model helper has a separate ID14 semantic parser and object_count+1 rule; that model behavior is preserved as such in the companion model/native-BSI package. It is not a native requirement for the selected outgoing path.

The path conclusion is bounded to the selected module, candidate set and recovered call chain. It does not claim there is no parser anywhere in the DLL, another module, another profile, an arbitrary indirect entry or a decoder. No native malformed-ID14 vectors were run because the recovered path contains no such semantic-reader invocation.

The four same-parent serializer lineage rows in `SELECTED-PATH-READER-STATUS.json` are rehashed prior R14 observations, not new dynamic capture. They prove producer/queue/serializer descriptor identity continuity for those rows; they are not malformed parser tests.

Whole F13C remains NOT_READY: semantic carrier_block mapping, production-cold equivalence, full-parent failure/publication and product acceptance remain open. The package contains no DLL, raw disassembly, private key, authentication value or PCM.
