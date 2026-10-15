# Phase 1 计划

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

| task_id | functional_scope | covered_clauses | planned_files | planned_tests | status |
|---|---|---|---|---|---|
| CLOCK-FOUNDATION | {"F02": "SCOPED_CLAUSES_ONLY", "XSTJ": "SCOPED_CLAUSES_ONLY"} | `F01/F02 input sample-count clauses`<br>`callback g`<br>`codec source s=g-2`<br>`JOC dispatch d=s+1=g-1`<br>`generated JOC writer=d-1=s`<br>`source0/1 suppression`<br>`source s>=2 to published AU=s-2`<br>`published AU ordinary/JOC writer offsets`<br>`programme count arithmetic` | `crates/jocforge-domain/src/timeline/index.rs`<br>`crates/jocforge-domain/src/timeline/clock.rs`<br>`crates/jocforge-domain/src/timeline/mod.rs` | `crates/jocforge-domain/tests/timeline_clock_contract.rs` | LOCALLY_VERIFIED |
| SELECTED-PROFILE-GATE | {} | — | `crates/jocforge-domain/src/profile/mod.rs`<br>`crates/jocforge-domain/src/profile/production.rs`<br>`crates/jocforge-domain/src/profile/validation.rs` | `crates/jocforge-domain/tests/selected_profile_gate.rs` | LOCALLY_VERIFIED |

## Failing-first tests

1. compile-fail cross-domain newtype interchange
2. g=0/1 source underflow rejection
3. s=0/1 suppression
4. s=2 to AU0
5. g-dispatch-generated-writer boundaries
6. sample counts 1,1535,1536,1537,6522906
7. 4247 normal plus 4 drain
8. 4249 sources
9. 4247 published AUs
10. ordinary/JOC publication offsets
11. pending ordinary 4247/4248 and JOC 4248
12. callback 4250 source/writer relation

## P1.2-A consumer-driven numeric slice

```json
{
  "consumer": {
    "functional_id": "F03P",
    "module": "crates/jocforge-domain/src/spatial/point_response.rs",
    "scope": "F03P.2 Q15 coordinate quantization only"
  },
  "implementation_status": "LOCALLY_VERIFIED",
  "planned_files": [
    "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "crates/jocforge-domain/src/numeric/mod.rs"
  ],
  "planned_tests": [
    "crates/jocforge-domain/tests/numeric_q15_coordinate.rs"
  ],
  "primitive": {
    "input": "finite f32",
    "name": "quantize_q15_coordinate",
    "negative_zero": "preserve through multiply/add semantics; truncates to integer zero",
    "non_finite": "reject input or intermediate NaN/Infinity",
    "output": "Result<Q15Coordinate, Q15CoordinateError>",
    "overflow": "reject if final truncated value cannot fit i32",
    "rounding": "binary32 round-to-nearest ties-to-even at multiply and add",
    "saturation": "positive values above 32767 saturate to 32767; no negative clamp",
    "shift": "NOT_APPLICABLE",
    "typed_value": {
      "accessor": "pub const fn value(self) -> i32",
      "mutability": "immutable",
      "name": "Q15Coordinate",
      "representation": "private i32",
      "scope": "F03P.2 scoped typed value"
    }
  },
  "spec_binding": {
    "file": "specs/03-point-response-and-target-context.md",
    "numeric_file": "specs/00-numeric-semantics.md",
    "numeric_sections": [
      "N00.1",
      "N00.2"
    ],
    "section": "F03P.2 15-point basis and quantization"
  },
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.2-A-Q15-COORDINATE",
  "vectors": {
    "case_pointers": [
      "U-PROJ-AXES",
      "U-PROJ-ZERO"
    ],
    "file": "test-vectors/unit/vectors.json",
    "kind": "PARENT_INTEGRATION_VECTOR"
  }
}
```
