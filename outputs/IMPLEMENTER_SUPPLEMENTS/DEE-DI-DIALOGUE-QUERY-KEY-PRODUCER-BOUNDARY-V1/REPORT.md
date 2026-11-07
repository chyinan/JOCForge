# Dialogue Intelligence getter query-key producer boundary

## Bound receiver and query path

The selected ParametersContainer getter callsite receives the same static Priv+0xec0 object previously bound to the constructor-built candidate list. Its query argument is a current linked-record string created by a record-flattening helper from input records supplied through the dispatcher's param_2.

The helper walks dynamic 0x68-byte records, reads a name-like field at +0x18 and nested metadata beginning at +0x38, and recursively flattens nested collections. The empty-input path seeds a linked record from supplied prefix/name state. Actual branch selection and string bytes are not captured.

## First break

The owner of the helper's input record array and the selected record-name bytes are not statically joined to the registered dialogue_intelligence key. The earlier exact-key-to-Bool binding means that supplying that exact key would select the Bool candidate; it does not prove that this callsite supplies it. No parse or audio/JOC consumer was followed.

The upstream 16-file package manifest SHA-256 is b39697204309881127d896f8d112523a4b2aeaa8889d92610a8d5ae968df0bec and its package-seal file SHA-256 is 2ffab90fcbceb7a4e598b0e1ee7c09a7292914dcbfd779613d93cbdde029f320. Its checker passed normally and under python -O. No raw Ghidra slices, media, PCM, payload, key, or license are included.
