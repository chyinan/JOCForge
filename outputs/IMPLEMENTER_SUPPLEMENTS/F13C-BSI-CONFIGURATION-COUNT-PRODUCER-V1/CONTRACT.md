# Selected BSI configuration-count producer

Status: `CONFIGURATION_COUNT_PATH_CLOSED_CAPTURE_LOCAL; SOURCE_GENERATION_UNBOUND`.

## Bounded behavior

For the reviewed ordinary, non-bypass JOC wrapper at 48 kHz and 1024 kbps, the hash-bound capture used `rate_code=5` and 38 signed I16 channel labels. The selected configuration recipe counted 15 non-LFE channels and one LFE-presence flag:

```text
configured_count_i32 = non_lfe_count + lfe_presence = 15 + 1 = 16
```

The selected parameter query used parameter 13 as a signed I32 with selector 10 and returned 16. Within that capture, the query result was passed to a configuration-record constructor; the selected BSI producer consumed the resulting configuration record under one capture-local owner and emitted a record that the importer subsequently consumed.

An existing hash-bound wire-field supplement identifies the destination as `complexity_index_type_a`, an 8-bit field beginning at frame bit 105 (cursor 113 afterward), corresponding to working-header byte offset 119. This field location does not bind the captured record to a particular source frame or published AU.

## Relationship to ID14

The bounded BSI count producer is sourced from wrapper/configuration channel-label count and LFE presence. It does not consume semantic ID14 `object_count` in the reviewed producer path. Bounded calls with validated configuration counts 1 and 16 succeeded while the ID14 queue/body, EMDF serializer, and JOC-body holder were absent. This establishes only that the bounded producer calls did not require those ID14 inputs; it is not a whole-encoder or global parser-absence claim.

For the selected values, the clean model's `object_count + 1` also evaluates to 16. That numeric equality does not identify the two producers or prove that either value belongs to the same source generation.

## Relationship to the current Rust seam

The implementation agent reports that commit `9d483e3` added a clean-model seam which derives `SelectedComplexityIndexTypeA` from validated semantic `object_count`, then writes that typed U8 at bit 105. This package does not establish a correspondence between that Rust input/return type and the native configuration-record/BSI-importer record described above. In particular, it does not prove that the native record is the input to `joc::model_complexity::derive_model_joc_complexity` or `joc::additional_bsi`, nor that both represent the same selected source generation. Keep the Rust clean-model path separate until an identity join is established.

## Open identity joins

The wrapper scalar-origin observation and parameter-query observation both contain 16, but have no shared owner or source-generation token. Configuration, owner, and record identifiers in the BSI capture are local aliases; they are not stable IDs across captures. The separately observed natural configuration-import owner and its outer generations are not joined to the BSI capture's owner/configuration/record.

Therefore the following remain `UNBOUND`:

- wrapper scalar origin to the parameter query in the same source generation;
- BSI record to the selected natural encoder owner;
- BSI record to a selected source/frame/AU generation;
- whether the later F12 writer for the selected input consumes this exact record.

Do not treat this capture as a production rule for every AU, replace the existing clean-model mapping based only on the value 16, or infer a source-generation join from event order or matching values. The full F13C chain remains not ready.
