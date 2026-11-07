# Dialogue Intelligence measure-only paired results

## Controlled setup

Two authorized ADM inputs were each measured with `1770-4 / measure_only`, `speech_threshold=15`, and paired XML jobs differing only in `dialogue_intelligence`. Pair-normalization is recorded as equal after changing only that boolean. The runs used the normal DEE launch interface; no license argument/path/content was recorded or inspected.

## Results from the sealed receipt

| Input | DI | Gating | Final integrated LUFS | Speech % | Dialogue LUFS | True peak |
|---|---:|---|---:|---:|---:|---:|
| `human_is_cat` | false | level | -18.605364 | -1.00 | -120.00 | -5.47 |
| `human_is_cat` | true | level | -18.605364 | 0.00 | -120.00 | -5.47 |
| `my_jealousy` | false | level | -16.291187 | -1.00 | -120.00 | -2.01 |
| `my_jealousy` | true | dialogue | -24.719242 | 91.47 | -24.72 | -2.01 |

These are DEE measurement scalars from the receipt. For `human_is_cat`, only the speech-percentage sentinel changed in the reported fields. For `my_jealousy`, DI=true selected dialogue gating and changed final integrated loudness/dialogue loudness. This demonstrates a multi-input measurement-path effect; it does **not** prove a decoded PCM difference, encoded E-AC-3/JOC BSI change, or audible output change.

The receipt marks measurement and encoder passes complete, but the temporary output was MP4/M4A and was deleted. No DDP/JOC output or PCM was retained or compared. Numeric OS exit codes were not captured. Source ADM hashes were inherited from an earlier authorized receipt and were not recomputed in this run.

## Implementation scope

This supports implementing the measured DI behavior in the `1770-4 measure_only` path within the tested samples/configuration. It does not define the speech classifier, establish all ADM behavior, or specify how these results feed DRC/encoding. That downstream consumer remains a separate dependency.

## Provenance and limits

- Current receipt SHA-256: `5fde217056781608cf43dff97fc4d10baf95e815af98a1cef70bc863694f168d`.
- Receipt SHA sidecar matched the receipt bytes.
- Prior source-binding receipt SHA-256: `2a0c63814d6d0783f5736bcfd8232ff1038168ba9cf3fea2461512b831a7ec6e`; source files were not rehashed for this paired run.
- Temporary MP4/M4A, logs, ASCII input copies, job XML, and working directories are reported deleted by the receipt.
- No raw logs, PCM, EC3 payload, key, or license content are included here.
