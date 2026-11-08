# RF DRC profile consumer boundary

## Scope

This supplement records the conditional static mapping from the RF DRC profile token to a DDP encoder parameter, plus a separate native DRC table producer/bitstream consumer path. It does not establish that the XML value reaches the handler, that the two paths share an owner or generation, or that any profile changes encoded or audible output.

## Conditional token mapping

The RF property is a StringType with schema default film_light and six declared tokens. A scoped static lookup maps the token to a workflow U32 and conditionally maps it to DDPI_ENC_DRCPROF parameter 0x0f, but only if the RF handler receives the token. The DDPJoc config record pointer is at sibling +0xd8 and the workflow U32 at sibling +0xe0.

The input-only table lists the six observed static pairs. Those pairs are not runtime XML admission, selected-handler, or output observations. The XML writer into the canonical StringType value member +0x08, provider/handler selection, and owner/generation remain UNBOUND.

## Separate DRC consumer path

Ghidra 12.2 static evidence identifies an encoder-core selector at state+0xac8 (U8), with a custom-profile flag at state+0xac4 (I32). The selector is copied to +0x21ab0 and a helper resolves a curve into context at +0x21aa8. A separate DRC table producer writes per-block U16 code and presence rows; two bitstream consumers read those rows and serialize one-bit presence followed by an optional eight-bit code.

No typed join connects RF parameter 0x0f or the StringType to this selector, table owner/mode, or generation. The table path is therefore a separate static path, not proof of RF profile behavior.

## BSI and runtime boundaries

The bounded crosswalk distinguishes header compre presence (U1) from optional compr value (U8). This slice did not bind a native writer for either field to parameter 0x0f. The prior N38 BSI observation was not reopened or parsed again; no BSI values are included here. No DEE run or dynamic capture occurred.

## Handoff

IMPLEMENTATION_READY=NO. Preserve the first break at the runtime XML/property node to the same-owner StringType value writer at +0x08. Keep the RF-to-core DRC selector/table join and native compre/compr writer UNBOUND. Runtime remains NOT_RUN; no claim of audible or PCM effect is made.
