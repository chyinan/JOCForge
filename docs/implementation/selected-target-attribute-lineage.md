# Selected target attribute lineage

This document maps washed F03G behavior for the selected target path. It does not implement the target assignment transition or make F03G production-ready.

## Evidence identity

- Washed package: `selected-target-attribute-lineage-washed-v1`.
- Original package manifest SHA-256: `207c9f33d2ec921c7a13c90f015133aba543edde8f88a557b71acb659073568f`.
- Washed specification SHA-256: `29177b0bfc6bdd4f1d7e641c53fa6afafd4be238e4df383cc964c808fd236867`.
- Washed relation vectors SHA-256: `dde8e489895978087eca37c070acc30220b805984fa3924fa17f521bbf868408`.
- Bounded-coverage SHA-256: `42dbb6ee931248f3ab4bb3a96a534bc6922779e552d537f63e03558902b8d6e1`.
- Local relation fixture preserves the washed vector bytes exactly. The local manifest and independent-check files are line-ending-normalized mirrors; their `manifest_sha256` identifies the original package manifest above.

The package reports `production_usable=false` and `IMPLEMENTATION_READY=NO`. These facts are recorded as package readiness, not as a user distribution restriction or a conclusion that the data is vendor-private.

## Selected-path behavior

The selected path has a target capacity of 15. Cold construction initializes current and previous IDs to `0xffffffff`, current and previous attributes to `0x00000B00`, and active and previous-marker-active flags to false for all slots. The existing F03 cold-provider seam is locally checked against these relations.

The washed runtime capture uses six new entries, all with attribute `0x00000B00`. It reports no shortage, activates the first six slots, and leaves remaining current attributes unchanged.

The observed packer for five integer components is:

```text
pack(a1, a2, a3, a4, a5) =
    a1 | 2 * (a2 | 8 * (a3 | 16 * (a4 | 4 * a5)))
```

The cold components produce `0x00000B00`. Their independent type-width and overflow contract is not established. Do not infer one from the formula.

For each candidate row/slot pair, the washed cost relation is:

```text
squared_euclidean_distance
+ 20.0 * id_mismatch
+ 100.0 * attribute_mismatch
+ 50.0 * multiplicity_balance_mask
```

The multiplicity term is conditional. The shortage comparator checks whether selected-count exceeds retained-count. A true shortage continues into assignment and is not a fatal `UNSUPPORTED_ATTRIBUTE_MULTIPLICITY` outcome. In the selected path, the extra mask is 1 when new and old attributes are equal, otherwise 0.

Assignment repeatedly selects the global minimum. A candidate replaces the current best only when strictly better; ties retain ascending row, then ascending column order. This is greedy assignment, not a Hungarian/global-optimal solver.

For mapped slots, previous XYZ/ID/attribute receive the old current values, the selected values become current, and active becomes true. For unmapped slots, active becomes false while current and previous fields remain unchanged.

## Verification boundary

| Behavior | Washed evidence | Local status |
| --- | --- | --- |
| Cold capacity, sentinel IDs, default attributes, cleared flags | Selected-path runtime confirmed | Relation checked against the existing F03 cold provider |
| Six-entry all-equal-attribute selection and selected-path projection | Selected-path runtime confirmed | No F03G transition exists in this repository; not locally executed |
| Mixed-attribute shortage continues without an unsupported-multiplicity error | Static evidence only | `STATIC_ONLY_NOT_RUN` |
| Multiplicity-mask equality rule and cost contribution | Static evidence only for the mixed branch | `STATIC_ONLY_NOT_RUN` |
| Greedy tie order | Washed relation vector, not a local execution vector | `STATIC_ONLY_NOT_RUN` |
| Attribute packer width and overflow behavior | Not independently closed | Do not implement from inference |

The relation tests verify the existing cold-provider seam and preserve the washed package's non-ready status. They do not execute or validate an assignment algorithm. Other dispatch modes, capacities, configurations, fault injection, reset, serialization, digest, receipt, transaction, and product behavior remain outside this contract.
