# Custom 5.1 trim: first consumer boundary

## Scope

This supplement covers only the schema choices and static StringType metadata for surround_trim_5_1 and height_trim_5_1, plus the bounded candidate scans described below. It does not define numeric trim values or implement gain behavior.

## Static facts

- Both properties are declared as xs:string choices and registered as dee::Types::StringType members by FUN_180136220: surround at param_1+0x968 and height at param_1+0x9e8.
- Their constructor literals are auto. This binds a constructor default only; XML token assignment to the selected runtime instance and omitted-field materialization remain UNBOUND.
- The XML strings do not provide a numeric unit, numeric precision, or value conversion rule. The input-only vectors list the declared tokens but do not assign numeric meaning to them.
- For surround, a bounded candidate FUN_1800b5c60 reads a U16 from [RDI+0x968] and writes a U16 to [RCX+0x240]. RDI is not joined to the registered StringType object or selected runtime owner. This is a candidate data path, not a demonstrated audio-gain consumer.
- For height, the scoped +0x9e8 query found initialization and a lifecycle cleanup path, but no value reader in that query scope. This is not a global absence claim.

## Open boundary

The first effective audio/object-gain consumer is UNBOUND for both fields. Token assignment, auto-selection input, numeric conversion, units, precision, BSI mapping, and audible effect are UNBOUND or NOT_RUN.

The reused dynamic contrast of surround auto versus -3 with height held at auto is PROPOSED_NOT_RUN. It has no launch authorization in this supplement. No XSD/path preflight was repeated in the source slice.

## Handoff status

IMPLEMENTATION_READY=NO. Do not treat the constructor auto literal as runtime default behavior or the surround candidate function as an audio consumer. No DEE/JOCForge run, media/PCM/payload/key/license read, or implementation contact occurred for this slice. The package contains no product source or decompilation.
