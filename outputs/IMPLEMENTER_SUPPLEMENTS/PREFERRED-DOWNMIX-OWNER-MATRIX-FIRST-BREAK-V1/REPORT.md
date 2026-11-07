# Preferred-downmix owner to matrix/PCM boundary

## Disposition

This addendum reuses the hash-bound DME host/argv, native field parser, level lineage, and Ghidra 12.2 option-owner receipts. It follows one selected encode-pass edge from the correct typed MixMode owner and stops at the first child that receives only the owner pointer. No actual channel-matrix or PCM consumer is established on this edge.

The DDP Atmos plugin owner is `dee::proc::Priv`. Its `MixMode` subobject is at `Priv+0x930`; the typed `MixMode+0x50` code is at `Priv+0x980`. In selected pass `FUN_1801475f0`, the exact `Priv+0x980` check found zero instruction references and zero decompiler references. At callsite `0x180147acf`, the pass calls `FUN_1801535b0` and passes the containing `Priv` pointer, not a scalar read from the mode field. The child body also has zero direct reads of `Priv+0x980`. Its first already-bound member path is the distinct `Priv+0x210` StringType path; it is not a matrix/PCM consumer. The first break is therefore `Priv+0x980` value -> selected child field read, before any identified audio transform.

The observed getter path is only formatting: `FUN_180171f50` reads the typed MixMode value and its sole direct caller `FUN_180171ce0` formats it as a token. No direct caller of that formatter or audio/BSI usepoint is bound in the reused receipt. This does not classify the option as signaling-only.

The four LoRo/LtRt level keys are a separate chain in the `dee_ddpjoc_encoder.exe` image. Native parser helper `FUN_1404de3d0` writes helper-shaped values to `puVar16[0xf]`, `[0x10]`, `[0x12]`, and `[0x13]`; typed level descriptors and first typed consumers remain `UNBOUND`. The host keys (`loro_cmix`, `loro_smix`, `ltrt_cmix`, `ltrt_smix`) and native XML keys (`lorocmixlev`, `lorosurmixlev`, `ltrtcmixlev`, `ltrtsurmixlev`) have no verified adapter/owner-generation join to the plugin's MixMode field.

A separate callback getter route reads doubles at callback-object `+0x58/+0x60` and passes them to `FUN_1404813f0`, which writes derived fields at `+0xf0/+0xf4/+0xf8`. Existing evidence does not join those doubles to the four XML helper-result slots or to `Priv+0x980`; it is not evidence of a channel matrix or PCM consumer.

## Classification

| Effect question | Status |
|---|---|
| Preferred-downmix host value to native typed MixMode owner | `UNBOUND` |
| Four LoRo/LtRt level result slots to a typed level owner | `UNBOUND` |
| MixMode value read by the selected encode pass/child | No read on the inspected direct edge |
| First actual channel matrix or PCM consumer | `UNBOUND` |
| BSI/signaling-only effect | `UNBOUND` |
| Sample-transform effect | `UNBOUND` |
| Intentional ignore | `UNBOUND` |

The validator for the separate native `dmixmod` XML key is a presence/string allowlist check. It does not bind the host `preferred_downmix_mode` string to the typed MixMode owner. Do not infer a mapping from `loro/ltrt` labels, numeric equality, neighboring fields, or output-field names.

## Scope

The referenced packages were hash-checked from the existing research workspace. Their static Ghidra traces use Ghidra 12.2 in read-only/no-analysis mode. No new binary-wide search was run, and no IDA, Everything, DEE, media, PCM, payload, key, or license content was accessed. No implementation repository was edited or contacted. This report makes no product-wide no-consumer claim.
