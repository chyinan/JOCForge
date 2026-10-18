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

## P1.2-B finite R32 add/multiply slice

```json
{
  "consumer": {
    "functional_id": "F03P",
    "module": "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "scope": "F03P.2 multiply and signed-half add rounding points only"
  },
  "implementation_status": "LOCALLY_VERIFIED",
  "planned_files": [
    "crates/jocforge-domain/src/numeric/finite_f32.rs",
    "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "crates/jocforge-domain/src/numeric/mod.rs"
  ],
  "planned_tests": [
    "crates/jocforge-domain/tests/numeric_r32_primitives.rs",
    "crates/jocforge-domain/tests/numeric_q15_coordinate.rs"
  ],
  "primitives": [
    {
      "accessors": [
        "pub const fn value(self) -> f32",
        "pub const fn to_bits(self) -> u32"
      ],
      "name": "FiniteF32",
      "representation": "private f32",
      "semantics": [
        "reject NaN/Infinity",
        "preserve positive/negative zero bits",
        "preserve finite subnormal bits"
      ],
      "signature": "FiniteF32::new(f32) -> Result<FiniteF32, FiniteF32Error>"
    },
    {
      "fusion": "FORBIDDEN",
      "name": "r32_mul",
      "overflow": "reject non-finite result",
      "rounding": "one binary32 round-to-nearest ties-to-even point",
      "signature": "r32_mul(FiniteF32, FiniteF32) -> Result<FiniteF32, R32Error>",
      "signed_zero": "input signs XOR"
    },
    {
      "fusion": "FORBIDDEN",
      "name": "r32_add",
      "overflow": "reject non-finite result",
      "rounding": "one binary32 round-to-nearest ties-to-even point",
      "signature": "r32_add(FiniteF32, FiniteF32) -> Result<FiniteF32, R32Error>",
      "signed_zero": "exact cancellation and opposite zero produce +0; same-sign zeros preserve sign"
    }
  ],
  "spec_binding": {
    "consumer_file": "specs/03-point-response-and-target-context.md",
    "consumer_section": "F03P.2 15-point basis and quantization",
    "file": "specs/00-numeric-semantics.md",
    "sections": [
      "N00.1 primitives",
      "N00.2 boundaries"
    ]
  },
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.2-B-FINITE-R32-ADD-MUL",
  "vectors": {
    "case_pointers": [
      "U-PROJ-AXES",
      "U-PROJ-ZERO"
    ],
    "file": "test-vectors/unit/vectors.json",
    "kind": "PARENT_INTEGRATION_VECTOR",
    "property_binding": [
      "bit-pattern signed-zero cases",
      "finite subnormal preservation",
      "separate multiply then add differs from fused result"
    ]
  }
}
```

## P1.2-C finite F32 truncation slice

```json
{
  "consumer": {
    "functional_id": "F03P",
    "module": "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "scope": "F03P.2 biased finite F32 to I32 truncation only"
  },
  "implementation_status": "LOCALLY_VERIFIED",
  "planned_files": [
    "crates/jocforge-domain/src/numeric/trunc_i32.rs",
    "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "crates/jocforge-domain/src/numeric/mod.rs"
  ],
  "planned_tests": [
    "crates/jocforge-domain/tests/numeric_trunc_i32.rs",
    "crates/jocforge-domain/tests/numeric_q15_coordinate.rs"
  ],
  "primitive": {
    "name": "trunc_f32_to_i32",
    "non_finite": "unrepresentable at API because input is FiniteF32",
    "overflow": "reject truncated values outside I32 range",
    "rounding": "truncate toward zero; never rne_int",
    "saturation": "FORBIDDEN",
    "signature": "trunc_f32_to_i32(FiniteF32) -> Result<TruncatedI32, TruncI32Error>",
    "signed_zero": "positive and negative zero both produce integer zero",
    "subnormal": "all finite magnitudes below one produce integer zero",
    "typed_value": {
      "accessor": "pub const fn value(self) -> i32",
      "mutability": "immutable",
      "name": "TruncatedI32",
      "representation": "private i32"
    },
    "wrap": "FORBIDDEN"
  },
  "spec_binding": {
    "consumer_file": "specs/03-point-response-and-target-context.md",
    "consumer_section": "F03P.2 15-point basis and quantization",
    "file": "specs/00-numeric-semantics.md",
    "section": "N00.1 trunc toward zero"
  },
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.2-C-TRUNC-F32-TO-I32",
  "vectors": {
    "case_pointers": [
      "U-PROJ-AXES",
      "U-PROJ-ZERO"
    ],
    "file": "test-vectors/unit/vectors.json",
    "kind": "PARENT_INTEGRATION_VECTOR",
    "property_binding": [
      "fractional signs truncate toward zero",
      "signed zero and subnormal to zero",
      "I32 range rejection"
    ]
  }
}
```

## P1.2-D normalized Q15 coordinate slice

```json
{
  "consumer": {
    "functional_id": "F03P",
    "module": "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "scope": "F03P.2 quantized integer divided by 32768 and converted to F32 only"
  },
  "implementation_status": "LOCALLY_VERIFIED",
  "planned_files": [
    "crates/jocforge-domain/src/numeric/q15_coordinate.rs",
    "crates/jocforge-domain/src/numeric/mod.rs"
  ],
  "planned_tests": [
    "crates/jocforge-domain/tests/numeric_q15_normalization.rs",
    "crates/jocforge-domain/tests/numeric_q15_coordinate.rs"
  ],
  "primitive": {
    "conversion": "Q15 integer to binary32 round-to-nearest ties-to-even; exact for reachable Q15Coordinate values",
    "division": "binary32 divide by exact nonzero 32768.0; exact power-of-two scaling",
    "division_by_zero": "impossible; fixed divisor 32768",
    "multiply_reciprocal": "FORBIDDEN",
    "name": "normalize_q15_coordinate",
    "non_finite": "impossible for typed input and bounded power-of-two scaling",
    "overflow": "impossible; output range is finite",
    "signature": "normalize_q15_coordinate(Q15Coordinate) -> NormalizedQ15Coordinate",
    "signed_zero": "integer zero normalizes to positive zero",
    "subnormal": "impossible; smallest nonzero magnitude is 2^-15",
    "typed_input": {
      "construction": "private; only quantize_q15_coordinate",
      "exact_f32_reconstruction": "all reachable stored integers lie on the originating binary32 integer grid",
      "name": "Q15Coordinate",
      "range": "i32 with positive cap 32767; negative values are not clamped"
    },
    "typed_output": {
      "accessors": [
        "pub const fn value(self) -> f32",
        "pub const fn to_bits(self) -> u32"
      ],
      "mutability": "immutable",
      "name": "NormalizedQ15Coordinate",
      "representation": "private f32"
    }
  },
  "spec_binding": {
    "file": "specs/03-point-response-and-target-context.md",
    "numeric_file": "specs/00-numeric-semantics.md",
    "numeric_sections": [
      "N00.1 R32",
      "N00.2 boundaries"
    ],
    "section": "F03P.2 15-point basis and quantization"
  },
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.2-D-NORMALIZE-Q15-COORDINATE",
  "vectors": {
    "case_pointers": [
      "U-PROJ-AXES",
      "U-PROJ-ZERO"
    ],
    "file": "test-vectors/unit/vectors.json",
    "kind": "PARENT_INTEGRATION_VECTOR",
    "property_binding": [
      "zero and signed-zero source normalize to +0",
      "positive and negative Q15 boundaries",
      "negative no-clamp values below -1",
      "reachable integer/F32 exactness boundaries"
    ]
  }
}
```

## P1.2-E CurrentFeatures finite R32 subtraction primitive

```json
{
  "consumer": {
    "functional_id": "F03F",
    "module": "crates/jocforge-domain/src/spatial/current_features.rs",
    "scope": "subtraction primitive dependency only for F03F.2 FFT twiddles/butterflies and F03F.5 accept=1-slow_retain; producer remains NOT_STARTED"
  },
  "generic_operator_trait": "FORBIDDEN",
  "implementation_files": [
    "crates/jocforge-domain/src/numeric/finite_f32.rs",
    "crates/jocforge-domain/src/numeric/mod.rs"
  ],
  "implementation_status": "LOCALLY_VERIFIED",
  "primitive": {
    "definition": "N00 subtraction is one R32 addition after flipping the right operand sign",
    "name": "r32_sub",
    "non_finite_input": "not constructible through FiniteF32; FiniteF32::new rejects NaN and infinity",
    "overflow": "reject non-finite result as R32Error::NonFiniteResult",
    "rounding": "one binary32 round-to-nearest ties-to-even point; no F64 widening or FMA",
    "signature": "r32_sub(FiniteF32, FiniteF32) -> Result<FiniteF32, R32Error>",
    "signed_zero": "right-sign flip followed by r32_add; exact nonzero cancellation is +0; all four zero-sign pairs pinned",
    "subnormal": "preserve finite subnormal inputs/results; gradual underflow"
  },
  "required_runtime_operations": {
    "F03F.2": [
      "tr = (c * b.real) - (d * b.imag)",
      "ti = (c * b.imag) + (d * b.real)",
      "upper/lower butterfly differences"
    ],
    "F03F.5": [
      "accept = 1 - slow_retain"
    ],
    "existing_primitives": [
      "FiniteF32",
      "r32_add",
      "r32_mul"
    ],
    "sole_missing_primitive": "r32_sub"
  },
  "spec_binding": {
    "consumer_file": "specs/03-feature-processor-and-state.md",
    "consumer_sections": [
      "F03F.2 explicit 4096-point mathematical FFT",
      "F03F.5 per-source state update"
    ],
    "consumer_sha256": "dbd1cf259d2e40013910868d31e38909678d3af8add9d80bd59d15e8cc6e8edd",
    "numeric_file": "specs/00-numeric-semantics.md",
    "numeric_sections": [
      "N00.1 primitives",
      "N00.2 boundaries"
    ],
    "numeric_sha256": "53cb67d6c9ddced1fb909dda806bb478a2926a66f2e38cecaada6f6e95d4d135",
    "supplement_identity": "current-features-producer.math-route.v1",
    "supplement_sections": [
      "6.2 twiddle and butterfly arithmetic",
      "7 accept computation"
    ],
    "supplement_spec_file": "SPEC.md",
    "supplement_spec_sha256": "1c298e21d81a702f8298dabba3f28e01576c9602729f5746d6665c0becfdeaa5"
  },
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.2-E-FINITE-R32-SUB",
  "test_files": [
    "crates/jocforge-domain/tests/numeric_r32_primitives.rs"
  ],
  "vectors": {
    "case_pointers": [
      "r32_sub_follows_signed_zero_and_subnormal_semantics",
      "r32_sub_rounds_exact_midpoints_to_even_independently",
      "r32_sub_rejects_non_finite_overflow"
    ],
    "independent_rne_bits": [
      {
        "expected": "0x3F800000",
        "left": "0x3F800000",
        "right": "0xB3800000"
      },
      {
        "expected": "0x3F800002",
        "left": "0x3F800001",
        "right": "0xB3800000"
      },
      {
        "expected": "0xBF800000",
        "left": "0xBF800000",
        "right": "0x33800000"
      },
      {
        "expected": "0xBF800002",
        "left": "0xBF800001",
        "right": "0x33800000"
      }
    ],
    "kind": "INDEPENDENT_PRIMITIVE_KNOWN_ANSWERS",
    "property_binding": [
      "signed-zero result bits",
      "finite subnormal preservation",
      "exact cancellation to positive zero",
      "ties-to-even on positive and negative midpoints",
      "overflow rejection"
    ],
    "test_file": "crates/jocforge-domain/tests/numeric_r32_primitives.rs"
  }
}
```

## P1.2 scoped closure

```json
{
  "completed_functional_ids": [],
  "future_consumers": {
    "implementation_status": "DEFERRED_DETAILED_MAPPING",
    "inventory_status": "INVENTORIED",
    "reopen_rule": "each new consumer requires a new independent P1.2 slice with consumer/spec/vector/error binding before implementation"
  },
  "global_numeric_complete": false,
  "materialized_consumers": [
    {
      "clause": "F03P.2",
      "functional_id": "F03P"
    },
    {
      "clause": "F03F.2/F03F.5 numeric primitive dependency only; algorithm NOT_STARTED",
      "functional_id": "F03F"
    }
  ],
  "scope": "current materialized numeric consumers only",
  "status": "LOCALLY_VERIFIED",
  "verified_slices": [
    {
      "primitives": [
        "quantize_q15_coordinate"
      ],
      "slice": "P1.2-A"
    },
    {
      "primitives": [
        "FiniteF32",
        "r32_mul",
        "r32_add"
      ],
      "slice": "P1.2-B"
    },
    {
      "primitives": [
        "trunc_f32_to_i32",
        "TruncatedI32"
      ],
      "slice": "P1.2-C"
    },
    {
      "primitives": [
        "normalize_q15_coordinate",
        "NormalizedQ15Coordinate"
      ],
      "slice": "P1.2-D"
    },
    {
      "primitives": [
        "r32_sub"
      ],
      "slice": "P1.2-E"
    }
  ]
}
```

## P1.3-A reader_eof materialization audit

```json
{
  "programme_root": {
    "boundaries": {
      "domain": "crates/jocforge-domain/src/frontend/input_clock.rs owns pure transition and generated cold constructor",
      "io": [
        "crates/jocforge-io/src/bw64/mod.rs",
        "crates/jocforge-io/src/pcm/mod.rs"
      ],
      "io_rule": "seek/chunk/bounded read only; pass bytes and explicit offsets to domain; cannot mutate owner"
    },
    "canonical_serialization": {
      "f32": "exact U32 bits row-major",
      "field_order": [
        "callback_index",
        "source_cursor_samples",
        "normal_callbacks",
        "remaining_cached_drains",
        "last_normal_host38",
        "last_control_snapshot",
        "last_phase"
      ],
      "integers": "little-endian",
      "optional": "presence byte then payload",
      "padding": "none"
    },
    "clock": {
      "advance": "one per committed normal or cached-drain callback",
      "cold": 0,
      "type": "FrontendCallbackIndex"
    },
    "cold_provider": {
      "generation_specs": [
        "parameters/adm-installed48-state-v1.json",
        "specs/01-input-and-clock.md"
      ],
      "identity": "reader-eof-cold.selected-adm-installed48-state.v3",
      "kind": "GENERATED_PROVIDER",
      "provider_files": [],
      "rules": [
        "all integer clocks zero except remaining_cached_drains=4",
        "both optional caches absent",
        "last_phase=UNSET/0"
      ],
      "validation": [
        "schema version exact",
        "seven schema pointers resolve",
        "fixed array shapes 38x1536 and 38x13",
        "last_phase allowed 0/1/2",
        "remaining drains range 0..4"
      ]
    },
    "digest_provider": {
      "api": "sha2::{Digest, Sha256}",
      "crate": "sha2",
      "default_features": false,
      "purpose": "SHA-256 over canonical complete reader_eof bytes only",
      "version": "=0.10.9"
    },
    "evidence": [
      "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "test-vectors/GLOBAL-TRANSACTION-INPUT-VECTORS.json",
      "test-vectors/ADM-INSTALLED48-WHOLE-INPUT-VECTORS.json"
    ],
    "fatal": {
      "effect": "discard staged reader_eof and set programme RETIRED",
      "later_stages": "NOT_RUN",
      "retry": "full restart or destroy only"
    },
    "fields": [
      {
        "cold": 0,
        "name": "callback_index",
        "type": "U64LE"
      },
      {
        "cold": 0,
        "name": "source_cursor_samples",
        "type": "U64LE"
      },
      {
        "cold": 0,
        "name": "normal_callbacks",
        "type": "U64LE"
      },
      {
        "cold": 4,
        "name": "remaining_cached_drains",
        "type": "U32LE"
      },
      {
        "cold": null,
        "name": "last_normal_host38",
        "type": "Option<F32[38][1536]>"
      },
      {
        "cold": null,
        "name": "last_control_snapshot",
        "type": "Option<U32[38][13]>"
      },
      {
        "cold": 0,
        "name": "last_phase",
        "type": "U8 enum UNSET=0 NORMAL=1 CACHED_DRAIN=2"
      }
    ],
    "implementation_status": "LOCALLY_VERIFIED",
    "linear_capabilities": {
      "abort_reuse": "impossible after successful abort because staged token is moved",
      "prepared": {
        "clone": false,
        "consumed_by": "stage(self, prepared)",
        "type": "ReaderEofPrepared"
      },
      "staged": {
        "clone": false,
        "consumed_by": [
          "abort(self, staged)",
          "commit(&mut self, staged)",
          "fatal(&mut self, staged)"
        ],
        "type": "ReaderEofStaged"
      },
      "stale_token": "independently prepared stale token is consumed and rejected by generation/digest validation"
    },
    "machine_owner_semantics": {
      "clone": false,
      "commit_authority": "one machine instance owns committed root/generation/retired flag",
      "committed_access": "immutable &ReaderEofState only",
      "copy": false,
      "mutable_access": "none",
      "ownership": "linear single committed owner",
      "restart": "full_restart(self) consumes the unique machine",
      "type": "ReaderEofMachine"
    },
    "materialization_status": "MATERIALIZED",
    "permissions": {
      "abort": {
        "authority": "CallbackTransaction root",
        "effect": "discard staged reader_eof; committed bytes/generation unchanged"
      },
      "commit": {
        "authority": "CallbackTransaction root",
        "condition": "all callback stages and generation checks pass",
        "effect": "atomic root replacement; generation +1"
      },
      "prepare": {
        "authority": "F01R/F02",
        "inputs": [
          "committed reader_eof",
          "explicit source byte/sample offset",
          "validated host/control proposal"
        ],
        "mutation": "none",
        "output": "ReaderEofStagedProposal"
      },
      "stage": {
        "authority": "F01R/F02",
        "output": "complete staged reader_eof after-state"
      }
    },
    "receipt": {
      "canonical_digest": "SHA-256 over canonical serialized complete reader_eof bytes",
      "field_types": {
        "after_generation": "u64",
        "after_sha256": "ReaderEofDigest([u8;32]) / lowercase hex at schema boundary",
        "before_generation": "u64",
        "before_sha256": "ReaderEofDigest([u8;32]) / lowercase hex at schema boundary",
        "changed": "bool",
        "commit_count": "u8 constrained to 0/1",
        "owner_id": "ReaderEofOwnerId::ReaderEof",
        "transaction_id": "immutable String"
      },
      "generation": "cold=0; successful callback commit increments exactly once",
      "immutable_fields": [
        "owner_id=reader_eof",
        "before_generation",
        "before_sha256",
        "after_generation",
        "after_sha256",
        "changed",
        "commit_count",
        "transaction_id"
      ],
      "schema_file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "schema_pointer": "#/$defs/ownerReceipt",
      "serialization": "GLOBAL-TRANSACTION schema JSON object; key order is not semantic; no receipt-byte digest is claimed",
      "type": "OwnerReceipt<ReaderEof>"
    },
    "reset": {
      "authority": "programme-origin full restart",
      "effect": "destroy root and rerun generated cold constructor",
      "resume": "forbidden within retired attempt"
    },
    "root_id": "reader_eof",
    "schema_file": "parameters/adm-installed48-state-v1.json",
    "schema_pointers": [
      "/f03w_owned_fields/clocks/callback_index",
      "/f03w_owned_fields/clocks/source_cursor_samples",
      "/f03w_owned_fields/clocks/normal_callbacks",
      "/f03w_owned_fields/clocks/remaining_cached_drains",
      "/f03w_owned_fields/source_cache/last_normal_host38",
      "/f03w_owned_fields/source_cache/last_control_snapshot",
      "/f03w_owned_fields/source_cache/last_phase"
    ],
    "schema_version": "selected-adm-installed48-state.v3"
  },
  "task": {
    "implementation_status": "LOCALLY_VERIFIED",
    "programme_root": "reader_eof",
    "status": "LOCALLY_VERIFIED",
    "task_id": "P1.3-A-READER-EOF-LIFECYCLE"
  }
}
```

## P1.3-B frontend_control materialization audit

```json
{
  "programme_root": {
    "boundaries": {
      "domain": [
        "crates/jocforge-domain/src/frontend/config.rs",
        "crates/jocforge-domain/src/frontend/events.rs"
      ],
      "io": [
        "crates/jocforge-io/src/bw64/mod.rs"
      ],
      "io_rule": "I/O gathers bounded metadata bytes only; domain parses/validates and owns state"
    },
    "canonical_serialization": {
      "digest": "SHA-256 over complete canonical /value bytes",
      "encoding": "UTF-8 JSON",
      "ensure_ascii": false,
      "numbers": "JSON decimal integers; booleans/null lowercase",
      "separators": [
        ",",
        ":"
      ],
      "sort_keys": true,
      "trailing_bytes": "none"
    },
    "clock": {
      "advance": "one per committed normal or cached-drain callback",
      "cold": 0,
      "join_invariant": "must equal reader_eof callback_index in the same CallbackTransaction",
      "type": "FrontendCallbackIndex"
    },
    "cold_provider": {
      "file": "parameters/f01e-selected-constructor-v1.json",
      "identity": "F01E_SELECTED_COLD_V1",
      "kind": "FILE_PROVIDER",
      "manifest_sha256": "b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb",
      "shape": "11 top-level fields; objects[28] fixed records; optional snapshots 38x13 and 37x13",
      "validation": [
        "schema/version exact",
        "cold_preimage_hex decodes to canonical /value",
        "object ordinals exactly 0..27",
        "all optional bindings/snapshots null",
        "normalized_config exact"
      ]
    },
    "cross_root_coordination": {
      "atomicity_claim": "NOT_IMPLEMENTED; two independent machine commits must never be presented as global atomic success",
      "peer_root": "reader_eof",
      "required_checks": [
        "same transaction_id",
        "same callback_index",
        "both before generations current",
        "all callback roots staged before any root replacement"
      ],
      "required_future_interface": "CallbackJoinProposal { transaction_id, callback_index, reader_eof_before_generation, frontend_control_before_generation, reader_eof_staged, frontend_control_staged }",
      "same_global_transaction": true
    },
    "evidence": [
      "parameters/f01e-selected-constructor-v1.json",
      "parameters/adm-installed48-state-v1.json",
      "specs/01-frontend-config-dbmd-events.md",
      "specs/00-global-transaction-contract.md",
      "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json"
    ],
    "fatal": {
      "effect": "discard staged frontend_control and set programme RETIRED",
      "later_stages": "NOT_RUN",
      "retry": "full restart or destroy only"
    },
    "fields": [
      {
        "cold": "F01E_SELECTED_COLD_V1",
        "name": "schema",
        "type": "const UTF-8 string"
      },
      {
        "cold": 0,
        "name": "callback_index",
        "type": "U64"
      },
      {
        "cold": 0,
        "name": "normal_callbacks",
        "type": "U64"
      },
      {
        "cold": 4,
        "name": "remaining_cached_drains",
        "type": "U32"
      },
      {
        "cold": "BEFORE_FIRST_NORMAL",
        "name": "phase",
        "type": "enum string"
      },
      {
        "cold": "ordinal 0..27; host/event null; sampler_previous_present false; xyz_u32 [0,0,0]",
        "name": "objects",
        "type": "array[28] object records"
      },
      {
        "cold": null,
        "name": "last_control38_snapshot",
        "type": "Option<U32[38][13]>"
      },
      {
        "cold": null,
        "name": "last_regular37_snapshot",
        "type": "Option<U32[37][13]>"
      },
      {
        "cold": null,
        "name": "dbmd_binding",
        "type": "optional immutable DBMD identity/table"
      },
      {
        "cold": null,
        "name": "physical_binding",
        "type": "optional derived 10-bed/28-object/LFE/regular37 binding"
      },
      {
        "cold": {
          "bed_count": 10,
          "bit_rate_enum": 5,
          "bit_rate_kbps": 1024,
          "channels": 38,
          "dispatch_mode": 2,
          "event_start_samples": 0,
          "frame_samples": 1536,
          "layout_enum": 8,
          "line_drc": "none",
          "object_count": 28,
          "pcm_bits": 24,
          "rf_drc": "none",
          "sample_rate_hz": 48000,
          "source_samples": 6522906,
          "spatial_config_mode": 0,
          "target_capacity": 15,
          "time_base": "file_position",
          "virtual_prefix_count": 11
        },
        "name": "normalized_config",
        "type": "immutable selected config record"
      }
    ],
    "implementation_status": "P1.3_B1_LOCALLY_VERIFIED_STAGING_ONLY",
    "linear_capabilities": {
      "machine": {
        "clone": false,
        "copy": false,
        "ownership": "single committed owner",
        "type": "FrontendControlMachine"
      },
      "prepared": {
        "clone": false,
        "consumed_by": "stage(self, prepared)",
        "type": "FrontendControlPrepared"
      },
      "staged": {
        "clone": false,
        "consumed_by": [
          "global abort",
          "global commit",
          "global fatal"
        ],
        "type": "FrontendControlStaged"
      }
    },
    "materialization_status": "PARTIALLY_MATERIALIZED_NO_COMMIT",
    "permissions": {
      "abort": {
        "authority": "future global CallbackTransaction coordinator",
        "effect": "discard staged frontend_control; committed bytes/generation unchanged"
      },
      "commit": {
        "authority": "future global CallbackTransaction coordinator",
        "condition": "reader_eof and frontend_control share transaction_id/callback_index and all callback roots validate",
        "effect": "future atomic multi-root replacement; NOT implemented in P1.3-B audit"
      },
      "prepare": {
        "authority": "F01E pure config/event/control transition",
        "inputs": [
          "committed frontend_control",
          "validated ADM/config/DBMD/event bytes already gathered by I/O shell",
          "callback kind and reader_eof clock proposal"
        ],
        "mutation": "none",
        "output": "FrontendControlPrepared"
      },
      "stage": {
        "authority": "F01E",
        "output": "complete immutable FrontendControlStaged after-state"
      }
    },
    "prerequisite": {
      "implementation_prohibitions": [
        "no hardcoded provider value in P1.3-B",
        "no file I/O in frontend domain",
        "no serde_json or other JSON dependency before P1.4 approval",
        "no READY/LOCALLY_VERIFIED before prerequisite passes"
      ],
      "required_status": "LOCALLY_VERIFIED",
      "requirements": [
        {
          "name": "provider_identity",
          "value": "ProviderIdentity<F01E_SELECTED_COLD_V1>"
        },
        {
          "name": "manifest_sha256_pin",
          "value": "b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb"
        },
        {
          "name": "schema_validation",
          "value": "f01e-selected-constructor.v1 and /value pointer exact"
        },
        {
          "name": "json_validation",
          "value": "UTF-8 JSON parse plus exact shape/type/range validation"
        },
        {
          "name": "canonical_preimage",
          "value": "rebuild canonical bytes and compare cold_preimage_hex plus expected SHA-256"
        },
        {
          "name": "provider_receipt",
          "value": "immutable typed F01E provider receipt"
        },
        {
          "name": "boundary",
          "value": "io reads provider bytes; domain parses/validates bytes without file access"
        }
      ],
      "task": "P1.4-A-F01E-PROVIDER-CONTRACT"
    },
    "receipt": {
      "digest": "SHA-256 canonical complete frontend_control bytes",
      "generation": "cold=0; successful global callback commit increments exactly once",
      "immutable_fields": [
        "owner_id=frontend_control",
        "before_generation",
        "before_sha256",
        "after_generation",
        "after_sha256",
        "changed",
        "commit_count",
        "transaction_id"
      ],
      "schema_file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "schema_pointer": "#/$defs/ownerReceipt",
      "type": "OwnerReceipt<FrontendControl>"
    },
    "reset": {
      "authority": "programme-origin full restart",
      "effect": "destroy root and reload/validate exact cold provider",
      "resume": "forbidden within retired attempt"
    },
    "root_id": "frontend_control",
    "schema_file": "parameters/f01e-selected-constructor-v1.json",
    "schema_pointer": "/value",
    "schema_version": "f01e-selected-constructor.v1"
  },
  "task": {
    "cross_root_atomic_commit": "DEFERRED_NOT_IMPLEMENTED",
    "implementation_status": "P1.3_B1_LOCALLY_VERIFIED_STAGING_ONLY",
    "prerequisite": "P1.4-A-F01E-PROVIDER-CONTRACT LOCALLY_VERIFIED",
    "programme_root": "frontend_control",
    "status": "BLOCKED_ON_CROSS_ROOT_COORDINATOR",
    "task_id": "P1.3-B-FRONTEND-CONTROL-AUDIT"
  }
}
```

## P1.3-C two-root callback coordinator mapping

```json
{
  "abort_api": "one callback-level abort consumes the CallbackTransactionStaged value; every owner receipt has after=before, changed=false, commit_count=0; no owner state or generation changes",
  "blockers": [
    "the v2 CallbackTransactionReceipt must not be implemented until the missing owner roots and g>=2 source-stage roots are materialized",
    "MEASUREMENT, FINALIZE, and RESTART receipt semantics remain unresolved"
  ],
  "callback_owner_ids": [
    "analysis",
    "codec",
    "drc",
    "frontend_control",
    "integrity",
    "joc",
    "metadata",
    "meter",
    "output",
    "reader_eof",
    "spatial_audio",
    "spatial_control"
  ],
  "callback_owner_snapshot_semantics": "every CALLBACK receipt carries committed before/after snapshots for all 12 owners, including unchanged owners",
  "callback_receipt": {
    "ABORTED": "stage fatal sets programme RETIRED; all after values equal before; changed=false; commit_count=0; top_level_commit_count=0",
    "COMMITTED": "all 12 owner snapshots use actual committed-before and committed-after values; top_level_commit_count=1",
    "REJECTED": "preflight failure runs no stage and does not retire; all after values equal before; changed=false; commit_count=0; top_level_commit_count=0",
    "after_values": "actual committed-after snapshots only; discarded staged values never appear",
    "approved_wrapper_field_count": 13,
    "base_schema_file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
    "base_schema_identity": "selected-profile-transaction-receipt.v1",
    "base_schema_pointer": "#/$defs/transactionReceipt",
    "canonical_output_order": "owner_receipts sorted by owner_id ASCII ascending; sorting does not change DSP or commit order",
    "canonical_state_digests": {
      "all_owners": "each before_sha256/after_sha256 uses that owner's complete canonical state bytes",
      "receipt_wrapper": "no digest or HMAC; stable JSON encoding is a test format only, not execution authority or a DEE compatibility claim"
    },
    "commit_count_semantics": {
      "ABORTED_OR_REJECTED": "after values equal before; changed=false; commit_count=0",
      "COMMITTED": "changed owner=1; unchanged owner=0"
    },
    "fatal_receipt_rule": "fatal is present only on ABORTED; preflight REJECTED has no fatal receipt",
    "field_mapping_status": "RESOLVED_BY_WASH_ERRATUM",
    "input_validation": "accept arbitrary input array order, then require each known owner_id exactly once; reject missing, duplicate, or unknown IDs",
    "known_base_v1_field_count": 13,
    "owner_receipt_count": 12,
    "owner_receipt_fields": [
      "owner_id",
      "before_generation",
      "before_sha256",
      "after_generation",
      "after_sha256",
      "changed",
      "commit_count",
      "transaction_id"
    ],
    "owner_receipt_order": {
      "order": [
        "analysis",
        "codec",
        "drc",
        "frontend_control",
        "integrity",
        "joc",
        "metadata",
        "meter",
        "output",
        "reader_eof",
        "spatial_audio",
        "spatial_control"
      ],
      "status": "APPROVED_BY_WASH_SIDE_DESIGN"
    },
    "owner_receipt_owner_ids_ascii": [
      "analysis",
      "codec",
      "drc",
      "frontend_control",
      "integrity",
      "joc",
      "metadata",
      "meter",
      "output",
      "reader_eof",
      "spatial_audio",
      "spatial_control"
    ],
    "owner_receipt_semantics": "all owner transaction_id values equal the shared CallbackTransactionId; changed is true iff generation or state digest changes",
    "owner_schema_pointer": "#/$defs/ownerReceipt",
    "schema_identity": "selected-profile-transaction-receipt.v2",
    "top_level_commit_count": "1 for a successful CALLBACK; 0 for ABORTED or REJECTED",
    "type": "CallbackTransactionReceipt",
    "unidentified_v2_field": null,
    "wash_side_design_basis": {
      "authority": "approved internal API design choice; not a DEE/native behavior claim",
      "findings_sha256": "8eb3ddff2ca02caf0db5f9d6b2d126e586f9d91116efc2b9ed9f748ca5460f69",
      "input_only_vectors_sha256": "4cf2c0de3a69e8601b0c2e96cf734623316f2d8637dc6bfe7b2e9f5f164aa02a",
      "materials_verification": "PASS_RESEARCH_MATERIAL_STRUCTURE_AND_IDENTITIES_ONLY",
      "production_path_reference": null,
      "source_audit_sha256": "0bc4a6ebcbb9bc9df104cf725dd498c58f73657c3a7bf99027531d4ed6028eba",
      "whole_execution_and_product_execution": "NOT_RUN"
    },
    "wrapper_fields": [
      "schema",
      "transaction_id",
      "kind",
      "result",
      "attempt",
      "callback_index",
      "source_index",
      "dispatch",
      "au_index",
      "owner_receipts",
      "durable_append",
      "fatal",
      "top_level_commit_count"
    ],
    "wrapper_types": {
      "attempt": "u64",
      "au_index": "null unless s>=2; then j=s-2",
      "callback_index": "u64 for CALLBACK",
      "dispatch": "null when g<2; d=s+1 when g>=2",
      "durable_append": "null unless an approved AU publish append succeeded",
      "fatal": "nonnull only for ABORTED",
      "kind": "transaction kind enum; only CALLBACK semantics are approved",
      "owner_receipts": "exactly 12 unique OwnerReceipt values in canonical ASCII owner_id order",
      "result": "COMMITTED | ABORTED | REJECTED",
      "schema": "const selected-profile-transaction-receipt.v2",
      "source_index": "null when g<2; s=g-2 when g>=2",
      "top_level_commit_count": "COMMITTED=1; ABORTED/REJECTED=0",
      "transaction_id": "nonempty string; CallbackTransactionId"
    }
  },
  "callback_success_gates": {
    "dual_root_success": "FORBIDDEN",
    "g_0_1": "all four callback roots reader_eof, frontend_control, spatial_control, spatial_audio must succeed before the callback commits",
    "g_ge_2": "wait for every source root required by that callback before the callback commits",
    "internal_stages": "SOURCE, DISPATCH and AU append do not emit independent top-level success"
  },
  "commit_api": "CallbackTransactionCoordinator::commit(&mut self, staged: CallbackTransactionStaged) -> Result<CallbackTransactionReceipt, CallbackTransactionError>; staged contains fixed named owner fields, never a generic root list",
  "coordinator": {
    "dynamic_owner_list": false,
    "fixed_programme_owner_fields": [
      "reader_eof",
      "frontend_control",
      "spatial_control",
      "spatial_audio",
      "joc",
      "meter",
      "drc",
      "analysis",
      "codec",
      "metadata",
      "integrity",
      "output"
    ],
    "generic_root_trait": false,
    "macros": false,
    "programme_status_and_fatal": "owned by the fixed programme coordinator root",
    "retired_guard": "programme status RETIRED blocks every public callback path; root machines have no independently callable public commit/abort/fatal success API",
    "trait_registry": false,
    "type": "CallbackTransactionCoordinator"
  },
  "design_only": true,
  "fatal_api": "a stage fatal aborts the whole callback, records ABORTED plus the fatal receipt, sets programme status RETIRED, preserves all owner bytes/generations, and rejects subsequent calls",
  "implementation_rust": "FORBIDDEN until all fixed callback owner roots are materialized and integrated",
  "join_metadata": {
    "all staged callback values": [
      "CallbackTransactionId",
      "FrontendCallbackIndex"
    ],
    "each_owner_snapshot": [
      "owner_id",
      "before_generation",
      "before_sha256",
      "after_generation",
      "after_sha256",
      "changed",
      "commit_count",
      "transaction_id"
    ],
    "frontend_provider_lineage": [
      "package_manifest_sha256",
      "provider_file_sha256",
      "canonical_cold_sha256"
    ],
    "matching_rule": "the same nonempty CallbackTransactionId and FrontendCallbackIndex bind the callback and all owner snapshots"
  },
  "owner_receipt_count": 12,
  "owner_receipt_input_validation": "accept any input order, then require exactly 12 distinct known owner_id values and emit canonical ASCII order; reject missing, duplicate, or unknown IDs",
  "owner_receipt_order_rule": "fixed ASCII ascending owner_id order; order does not affect DSP stage or replacement order",
  "owner_root_readiness": {
    "additional_owner_roots_required_for_12_snapshots": [
      "joc",
      "meter",
      "drc",
      "analysis",
      "codec",
      "metadata",
      "integrity",
      "output"
    ],
    "currently_materialized_or_mapped": [
      "reader_eof",
      "frontend_control",
      "spatial_audio",
      "spatial_control"
    ],
    "g_ge_2_source_stage_roots": [
      "joc",
      "drc",
      "analysis",
      "codec",
      "metadata",
      "integrity",
      "output"
    ],
    "missing_callback_root_implementation": "spatial_audio",
    "spatial_audio_implementation_status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__DSP_PRODUCERS_DEFERRED",
    "spatial_audio_mapping_status": "MAPPING_COMPLETE_BLOCKED",
    "spatial_control_status": "MAPPING_COMPLETE_BLOCKED_ON_P1.3_C_ROOTS"
  },
  "preflight_order": [
    "validate programme status and callback g against the committed main callback clock",
    "validate nonempty CallbackTransactionId and matching callback_index for all present stage receipts",
    "require owner snapshot set to contain each of the 12 owner_id values exactly once; reject missing, duplicate, or unknown IDs",
    "validate all before_generation and before_sha256 values against the committed owner roots",
    "validate all after generations and after_sha256 values against fully prepared staged roots; require unchanged owners to carry after equal to before",
    "validate F01E and F03 provider lineages, plus all downstream owner receipts required by the selected callback",
    "checked-compute all next owner generations and final top-level receipt fields before replacement"
  ],
  "preflight_rejection": "result=REJECTED; no stage runs and programme status does not become RETIRED",
  "reader_eof_api_migration": "ReaderEofMachine commit/abort/fatal and per-root success APIs become crate-private coordinator implementation details; no root can independently publish success",
  "replacement": {
    "fallibility": "replacement phase contains only infallible moves and scalar assignments; no callbacks, allocation, hashing, cloning, or fallible work",
    "operation": "move the fixed named after-states into the programme owner fields and assign precomputed generations",
    "precondition": "all 12 owner checks, digest checks, lineage checks, and generation arithmetic finish before replacement begins",
    "rollback": "clone-and-rollback is forbidden; any preflight rejection leaves all 12 committed owner identities and generations unchanged"
  },
  "scope": "CALLBACK is the only top-level transaction in scope; SOURCE, DISPATCH, and AU append are internal stages; MEASUREMENT, FINALIZE, and RESTART remain unresolved",
  "status": "CALLBACK_CONTRACT_APPROVED_WAITING_OWNER_ROOTS",
  "success_semantics": "one CALLBACK top-level commit consumes the complete staged transaction, atomically replaces all 12 owner snapshots, and emits one immutable receipt; no per-owner or per-source top-level success exists",
  "task_id": "P1.3-C-CALLBACK-TRANSACTION-CONTRACT-MAP",
  "transaction_receipt_binding": {
    "field_bindings": {
      "attempt": "programme attempt generation",
      "au_index": "null unless s>=2; then j=s-2",
      "callback_index": "shared FrontendCallbackIndex",
      "dispatch": "null when g<2; d=s+1 when g>=2",
      "durable_append": "null unless an approved AU publish append succeeded; null does not prove that no physical write occurred",
      "fatal": "nonnull only for ABORTED; if durable append succeeded before root publication failed, record the abandoned physical append in fatal diagnostics",
      "kind": "CALLBACK; SOURCE, DISPATCH, and AU append are internal stages; MEASUREMENT, FINALIZE, and RESTART remain unresolved",
      "owner_receipts": "12 complete committed before/after snapshots, including unchanged owners, in the approved ASCII order",
      "result": "COMMITTED only after full callback replacement; preflight rejection is REJECTED; stage fatal is ABORTED",
      "schema": "selected-profile-transaction-receipt.v2",
      "source_index": "null when g<2; s=g-2 when g>=2",
      "top_level_commit_count": "1 for COMMITTED; 0 for ABORTED or REJECTED; SOURCE, DISPATCH, and AU append are internal stages",
      "transaction_id": "shared CallbackTransactionId"
    },
    "field_count_reconciliation": "RESOLVED_BY_WASH_ERRATUM; the callback wrapper has exactly 13 fields and no field 14",
    "field_names": [
      "schema",
      "transaction_id",
      "kind",
      "result",
      "attempt",
      "callback_index",
      "source_index",
      "dispatch",
      "au_index",
      "owner_receipts",
      "durable_append",
      "fatal",
      "top_level_commit_count"
    ],
    "json_stability": "stable JSON encoding is only a test format; no wrapper digest/HMAC or DEE compatibility claim"
  },
  "typed_transaction_id": {
    "representation": "private String",
    "type": "CallbackTransactionId",
    "validation": "nonempty, matching transaction_id minLength=1 in the GTC transactionReceipt schema"
  },
  "unclosed_transaction_kinds": [
    "MEASUREMENT",
    "FINALIZE",
    "RESTART"
  ],
  "vector_bindings": {
    "coverage": "committed-state preservation, callback/source counts, fatal retirement, and post-retired rejection; CALLBACK v2 field and twelve-owner receipt rules are supplied by wash-side design basis",
    "file": "test-vectors/GLOBAL-TRANSACTION-INPUT-VECTORS.json",
    "relevant_case_ids": [
      "continuous_main_prefix_through_au0",
      "callback_failure_after_f03_selection",
      "post_retired_calls_rejected",
      "full_restart_from_retired"
    ]
  },
  "wash_side_design_basis": {
    "authority": "approved internal CALLBACK API design choice; not a DEE/native behavior claim",
    "findings_sha256": "8eb3ddff2ca02caf0db5f9d6b2d126e586f9d91116efc2b9ed9f748ca5460f69",
    "input_only_vectors_sha256": "4cf2c0de3a69e8601b0c2e96cf734623316f2d8637dc6bfe7b2e9f5f164aa02a",
    "materials_verification": "PASS_RESEARCH_MATERIAL_STRUCTURE_AND_IDENTITIES_ONLY",
    "production_path_reference": null,
    "source_audit_sha256": "0bc4a6ebcbb9bc9df104cf725dd498c58f73657c3a7bf99027531d4ed6028eba",
    "whole_execution_and_product_execution": "NOT_RUN"
  }
}
```

## P1.3-D spatial_control root mapping audit

```json
{
  "programme_root": {
    "callback_local_only_fields": [
      "event_control.derived_control38",
      "event_control.derived_regular37",
      "feature.fft4096_work",
      "selection_outer.f03g_gate_output",
      "selection_outer.f03a_selector",
      "selection_outer.f03u_refinement",
      "selection_outer.f03n_nnls",
      "selection_outer.f03s_outer"
    ],
    "callback_output_packets": {
      "CurrentFeaturesPacket": {
        "bitwise_relation": "features[band][source] = source_vector_after[source][band]",
        "callback_index": "g",
        "cold_read_only_view": "may derive positive-zero transpose from source_vector; this does not mean callback 0 has completed",
        "cold_reset_read": "NOT_READY",
        "next_callback_seed": false,
        "order": "band-major",
        "payload": "features F32[24][48]",
        "persistent": false,
        "presence_tag": false,
        "producer_status": "LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED",
        "root_canonical_bytes": 0
      },
      "SourceFlagsPacket": {
        "callback_index": "g",
        "cold_reset_read": "NOT_READY",
        "derived_from": "same callback before history plus current input/control/route/prefix writer set",
        "implementation_file": "crates/jocforge-domain/src/spatial/source_flags.rs",
        "next_callback_seed": false,
        "payload": "flags U8[48][2]",
        "persistent": false,
        "presence_tag": false,
        "producer_status": "LOCALLY_VERIFIED_NUMERIC_AND_PACKET",
        "root_canonical_bytes": 0
      },
      "diagnostics": "packet copies may be recorded externally but are not recovery authority",
      "next_callback": "recompute flags from current before history and inputs; read features from committed source_vector/source_scalar, never from an older packet",
      "parent_failure": "invalidate packets and leave committed owner root unchanged",
      "parent_success": "commit only persistent histories; packet storage stays outside the root",
      "same_parent_transaction": "Both packets and their corresponding staged-after root belong to the same parent CallbackTransaction"
    },
    "canonical_state": {
      "arrays": "fixed arrays row-major",
      "digest": "SHA-256 over the complete serialized v5 persistent f03w_owned_fields; recompute from the full v5 preimage after callback outputs and immutable profile_dimensions are excluded",
      "external_state": "F05R56/F04C/F05JL/F13/F15 states remain under their own owners and are not embedded in this digest",
      "f32": "exact U32 bit patterns",
      "field_order": "canonical map in shown f03w_owned_fields order",
      "integers": "little-endian",
      "optional": "presence byte then payload",
      "scratch": "callback-local scratch is excluded from serialized committed state",
      "serialization_rule": "Canonical map in shown field order; fixed arrays row-major; integers little-endian; F32 exact U32 bits; optional values carry presence byte; variable records carry U32LE count and length. Callback-local scratch is never serialized. No pointers, padding, hash-expanded bytes or zero-filled unknown state.",
      "serialization_source": "parameters/adm-installed48-state-v1.json#/serialization",
      "variable_records": "U32LE count and length"
    },
    "clock": {
      "callback_index": {
        "cold": 0,
        "serialized": "U64LE",
        "type": "FrontendCallbackIndex"
      },
      "normal_callbacks": {
        "cold": 0,
        "serialized": "U64LE",
        "type": "NormalCallbackCount"
      },
      "owner_generation": "cold=0; increments once per successful enclosing main callback transaction",
      "owner_receipt_callback_generation": "must equal the enclosing CallbackTransaction callback index",
      "remaining_cached_drains": {
        "cold": 4,
        "serialized": "U32LE"
      },
      "retired": {
        "cold": false,
        "serialized": "U8 bool"
      },
      "source_cursor_samples": {
        "cold": 0,
        "serialized": "U64LE",
        "type": "U64 sample cursor"
      }
    },
    "cold_constructors": {
      "clocks": "all zero except remaining_cached_drains=4",
      "event_control": "parameters/f01e-selected-constructor-v1.json exact canonical cold",
      "feature": "source_vector F32[48][24] and source_scalar F32[48] all positive-zero bits 0x00000000",
      "matrix_assignment": "assignment=-1, markers=0, matrix positive zero, writer_set empty",
      "render_audio": "all numeric histories positive zero and no diffusion instances",
      "selection_outer": "parameters/f03-selected-target-cold-v1.json plus all declared scratch positive-zero/empty/index-zero constructors",
      "source_cache": "both optional caches absent; last_phase U8=0 UNSET"
    },
    "count_domains": {
      "committed_active_count": "derived sum of target_active U8[15]",
      "distinct_domains": [
        "candidate count",
        "selected count",
        "installed destinations",
        "outer-solver dimension",
        "committed active flags",
        "previous-marker flags",
        "NNLS working-set count"
      ],
      "persistent_state_field": false,
      "target_capacity_T": 15,
      "universal_active_count": false
    },
    "external_owner_refs": {
      "component_lfe_receiver_qmf": {
        "owner": "F04C",
        "receipt": "selected-j05-owner-receipt.v1",
        "state_schema": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json"
      },
      "joc_after_j05": {
        "cold_preimage": "test-vectors/JOC-OWNED-COLD-PREIMAGE.bin",
        "owner": "F05JL",
        "receipt": "immutable temporal/geometry/fit/huffman receipt",
        "state_schema": "test-vectors/JOC-OWNED-STATE-COMPLETE-SCHEMA.json"
      },
      "metadata_queues": {
        "owner": "F13",
        "receipt": "four-queue metadata receipt",
        "state_schema": "parameters/keyless-publication-state-v1.json"
      },
      "reader_target": {
        "owner": "F05R56",
        "receipt": "immutable reader56/target transition receipt",
        "state_schema": "test-vectors/READER56-TARGET-STATE-SCHEMA.json"
      },
      "source_publication": {
        "owner": "F15",
        "receipt": "source transaction/durable append receipt",
        "state_schema": "parameters/keyless-publication-state-v1.json"
      }
    },
    "f03w_owned_field_groups": {
      "clocks": [
        "callback_index",
        "source_cursor_samples",
        "normal_callbacks",
        "remaining_cached_drains",
        "retired"
      ],
      "event_control": [
        "owned_state",
        "derived_control38",
        "derived_regular37"
      ],
      "feature": [
        "source_vector",
        "source_scalar",
        "fft4096_work"
      ],
      "matrix_assignment": [
        "assignment",
        "markers",
        "matrix",
        "writer_set"
      ],
      "render_audio": [
        "regular_raw_history",
        "previous_pcm",
        "previous_current_rows",
        "diffusion_instances",
        "diffusion_previous_outputs"
      ],
      "selection_outer": [
        "target_current_xyz",
        "target_current_id",
        "target_current_attribute",
        "target_previous_xyz",
        "target_previous_id",
        "target_previous_attribute",
        "target_active",
        "previous_marker_active",
        "f03g_gate_output",
        "f03a_selector",
        "f03u_refinement",
        "f03n_nnls",
        "f03s_outer"
      ],
      "source_cache": [
        "last_normal_host38",
        "last_control_snapshot",
        "last_phase"
      ]
    },
    "failure_and_reset": {
      "failure_source": "parameters/adm-installed48-state-v1.json#/failure",
      "fatal": "preserve all owner roots' complete before bytes and generation; programme status becomes RETIRED with top-level commit_count=0",
      "reset": "destroy every owner root and call each referenced cold constructor; a missing constructor is fatal; unknown arrays are not zero-filled",
      "reset_source": "parameters/adm-installed48-state-v1.json#/reset"
    },
    "implementation_status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__PACKET_PRODUCERS_DEFERRED",
    "materialization_status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__PACKET_PRODUCERS_DEFERRED",
    "overlap_projection": {
      "commit_owner_labels": 6,
      "field_groups": 7,
      "file": "parameters/selected-owner-projection-v1.json",
      "receipt_owner_labels": 7,
      "role": "cross-owner overlap projection only; not used as the complete f03w_owned_fields schema"
    },
    "owner_receipt": {
      "commit_count": "1 when changed; 0 when unchanged, per GTC",
      "fields": [
        "owner_id=spatial_control",
        "before_generation",
        "before_sha256",
        "after_generation",
        "after_sha256",
        "changed",
        "commit_count",
        "transaction_id"
      ],
      "schema_file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "schema_pointer": "#/$defs/ownerReceipt",
      "serialization": "GTC JSON object; property order is not semantic and no receipt-byte digest is specified",
      "transaction_id": "same CallbackTransactionId as reader_eof and frontend_control",
      "type": "OwnerReceipt<SpatialControl>"
    },
    "profile_binding": {
      "active_target_count_n": {
        "canonical_member": false,
        "classification": "documentation_only",
        "cold_numeric_value": null,
        "numeric_state": false
      },
      "authorization": "reject cross-profile token binding even when state hashes are equal",
      "binding_sources": [
        "selected EncodingProfile identity",
        "validated F01E provider receipt",
        "validated F03 target provider receipt"
      ],
      "dimensions": {
        "axes": 3,
        "feature_bands_B": 24,
        "source_count_N": 48,
        "target_capacity_T": 15
      },
      "persistent_state_field": false,
      "presence_tag": false,
      "root_canonical_bytes": 0,
      "schema_identity": "selected-adm-installed48-state.v5"
    },
    "programme_owner_binding": {
      "layout_file": "parameters/adm-installed48-state-v1.json",
      "layout_manifest_entry_bytes": 7237,
      "layout_manifest_entry_sha256": "5138318de304b83d26e47271c1661bcc903cebcd2cea457ae98e0048b161ed46",
      "layout_pointer": "/f03w_owned_fields",
      "layout_schema": "selected-adm-installed48-state.v3",
      "owner_id": "spatial_control",
      "owner_mapping_basis": [
        "GTC owner table maps spatial_control to F03 feature, target current/previous, active/marker and main callback generation",
        "specs/03-adm-installed48-whole.md section 5 maps the F03W production root to f03w_owned_fields and lists external owners",
        "selected-owner-projection-v1.json is a seven-group cross-owner projection and is not the full owner schema"
      ],
      "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
      "previous_schema_amendment_file": "docs/implementation/CALLBACK-OUTPUT-PERSISTENCE-AMENDMENT.md",
      "schema_amendment_file": "docs/implementation/SPATIAL-CONTROL-STATE-V5-AMENDMENT.md",
      "state_ref_value": "parameters/adm-installed48-state-v1.json#/f03w_owned_fields",
      "superseding_logic_schema": "selected-adm-installed48-state.v5",
      "wrapper_bindings": {
        "cold_sha256": "SHA-256 over canonical bytes generated from the complete cold constructor and validated provider values",
        "generation": "main callback generation; cold=0",
        "owner_id": "spatial_control",
        "state_ref": "parameters/adm-installed48-state-v1.json#/f03w_owned_fields",
        "state_sha256": "SHA-256 over complete canonical f03w_owned_fields bytes"
      },
      "wrapper_fields": [
        "owner_id",
        "generation",
        "state_sha256",
        "cold_sha256",
        "state_ref"
      ],
      "wrapper_ref": "#/$defs/ownerState",
      "wrapper_schema_file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "wrapper_schema_pointer": "#/properties/owners/properties/spatial_control"
    },
    "provider_prerequisites": {
      "f01e_cold": {
        "file": "parameters/f01e-selected-constructor-v1.json",
        "receipt_lineage": "validated MANIFEST and provider entry receipt",
        "status": "P1.4-A-LOCALLY-VERIFIED"
      },
      "f03_target_cold": {
        "file": "parameters/f03-selected-target-cold-v1.json",
        "manifest_entry_bytes": 3139,
        "manifest_entry_sha256": "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00",
        "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
        "provider_kind": "exact_parseable_functional_constant",
        "required_prerequisite": {
          "required_before": "spatial_control implementation or provider-backed staging",
          "status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
          "task_id": "P1.4-B-F03-SELECTED-TARGET-COLD-PROVIDER-CONTRACT"
        }
      }
    },
    "readiness": {
      "blockers": [
        "P1.3-C callback implementation is waiting for all 12 owner roots, including spatial_audio and g>=2 source roots",
        "P1.3-D must not define any local root commit or top-level COMMITTED receipt"
      ],
      "implementation_status": "NOT_STARTED",
      "mapping_status": "MAPPING_COMPLETE_BLOCKED"
    },
    "root_id": "spatial_control",
    "stage_authority": {
      "commit_authority": "enclosing CallbackTransaction only; local spatial_control commit is forbidden",
      "owner": "F03W",
      "receipt": "SpatialControlReceipt",
      "receipt_fields": [
        "feature before/after",
        "selector/U04/outer scratch identity",
        "proposed target current/previous/active",
        "marker proposal and external F05R56 metadata-history receipt",
        "callback generation",
        "unique commit_count"
      ],
      "receipt_source": "specs/00-global-transaction-evidence-gaps.md#G02"
    },
    "transaction_stage": {
      "capabilities": [
        "SpatialControlPrepared non-Clone/non-Copy",
        "SpatialControlStaged non-Clone/non-Copy"
      ],
      "join_metadata": [
        "shared CallbackTransactionId",
        "same FrontendCallbackIndex as reader_eof and frontend_control",
        "spatial_control before_generation and before_sha256",
        "spatial_control after_sha256",
        "validated F01E provider lineage",
        "P1.4-B F03 target provider lineage",
        "external owner receipts"
      ],
      "local_commit": "FORBIDDEN",
      "status": "STAGING_ONLY_WAITING_ON_P1.3_C_OWNER_ROOTS",
      "top_level_committed": "FORBIDDEN"
    },
    "vector_evidence": {
      "digest_limit": "synthetic full_state_sha256 values and historical AI48ST01 reset payload are not production cold root digests",
      "files": [
        "test-vectors/ADM-INSTALLED48-WHOLE-INPUT-VECTORS.json",
        "test-vectors/F03-COLD-ATTRIBUTE-INPUT-VECTORS.json",
        "test-vectors/ADM-INSTALLED48-LOCAL-REVIEW-STATUS.json"
      ],
      "scope": "7-callback deterministic synthetic selected profile only; real ADM cold replay is NOT_RUN and package whole implementation readiness is NO"
    }
  },
  "task": {
    "blocked_on": [
      "spatial_audio render/audio receipt producer and integration",
      "enclosing CallbackTransaction coordinator"
    ],
    "blockers": [
      "CurrentFeatures math and capability production is locally verified; render/post-window audio production, spatial-audio root integration, and parent CallbackTransaction integration remain deferred",
      "P1.3-C callback implementation waits for all 12 owner roots, including spatial_audio and g>=2 source roots"
    ],
    "callback_local_scratch_not_serialized": [
      "event_control.derived_control38",
      "event_control.derived_regular37",
      "feature.fft4096_work",
      "selection_outer.f03g_gate_output",
      "selection_outer.f03a_selector",
      "selection_outer.f03u_refinement",
      "selection_outer.f03n_nnls",
      "selection_outer.f03s_outer"
    ],
    "canonical_state": "SHA-256 over the complete serialized f03w_owned_fields map in declared order; callback-local scratch and external owner states are excluded",
    "capabilities": [
      "SpatialControlPrepared non-Clone/non-Copy",
      "SpatialControlStaged non-Clone/non-Copy"
    ],
    "clock": "FrontendCallbackIndex main callback clock; source cursor U64 sample position; normal callback count; four cached drains; owner generation increments with the enclosing CallbackTransaction",
    "cold_providers": {
      "f01e_cold": {
        "file": "parameters/f01e-selected-constructor-v1.json",
        "status": "P1.4-A-LOCALLY-VERIFIED"
      },
      "f03_target_cold": {
        "bytes": 3139,
        "file": "parameters/f03-selected-target-cold-v1.json",
        "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
        "prerequisite_status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
        "sha256": "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00"
      }
    },
    "complete_owner_groups": [
      "clocks",
      "source_cache",
      "event_control",
      "render_audio",
      "feature",
      "selection_outer",
      "matrix_assignment"
    ],
    "external_owners_not_embedded": [
      "F05R56 reader_target",
      "F04C component_lfe_receiver_qmf",
      "F05JL joc_after_j05",
      "F13 metadata_queues",
      "F15 source_publication"
    ],
    "implementation_rust": "SourceFlagsPacket and CurrentFeatures math/capability producers are locally verified; full P1.3-D remains blocked on spatial_audio render integration and parent CallbackTransaction integration; D1 status is unchanged",
    "local_commit": "FORBIDDEN",
    "mapping_status": "MAPPING_COMPLETE_BLOCKED",
    "owner_receipt_order_and_wrapper": "P1.3-C approves 12-owner ASCII order and schema v2 with 13 fields; wrapper has no digest/HMAC; implementation waits for root availability",
    "packet_producer_blockers": [],
    "prerequisites": [
      {
        "status": "CALLBACK_CONTRACT_APPROVED_WAITING_OWNER_ROOTS",
        "task_id": "P1.3-C-CALLBACK-TRANSACTION-CONTRACT-MAP"
      },
      {
        "status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
        "task_id": "P1.4-B-F03-SELECTED-TARGET-COLD-PROVIDER-CONTRACT"
      }
    ],
    "previous_local_amendment": {
      "document_bytes": 4366,
      "document_sha256": "2e55a2e142b6482e456e40a9f3154f82574c2ed4259acab5ebab08369a84d474",
      "file": "docs/implementation/CALLBACK-OUTPUT-PERSISTENCE-AMENDMENT.md",
      "schema_id": "selected-adm-installed48-state.v4"
    },
    "programme_owner_schema": {
      "file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "owner_state_fields": [
        "owner_id",
        "generation",
        "state_sha256",
        "cold_sha256",
        "state_ref"
      ],
      "pointer": "#/properties/owners/properties/spatial_control",
      "ref": "#/$defs/ownerState"
    },
    "programme_root": "spatial_control",
    "provider_prerequisite": {
      "identity_source": "approved package MANIFEST.json entry",
      "required_provider": "parameters/f03-selected-target-cold-v1.json",
      "status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
      "task_id": "P1.4-B-F03-SELECTED-TARGET-COLD-PROVIDER-CONTRACT"
    },
    "receipt_order": "12-owner ASCII order approved by P1.3-C; v2 wrapper has no digest/HMAC and stable JSON is test-only",
    "root_audit": "source layout selected-adm-installed48-state.v3 remains bound to the approved package; local v4 and approved v5 amendments define the logical persistent root; callback outputs and immutable profile_dimensions are excluded from mutable state and seven persistent groups remain",
    "stage_authority": "F03W emits SpatialControlReceipt after all F03W leaf stages and downstream receipts pass",
    "stage_only": {
      "join_metadata": [
        "CallbackTransactionId",
        "FrontendCallbackIndex",
        "before_generation",
        "before_sha256",
        "after_sha256",
        "F01E provider lineage",
        "F03 target provider lineage",
        "external owner receipts"
      ],
      "linear": true,
      "local_commit": "FORBIDDEN",
      "prepared_type": "SpatialControlPrepared",
      "required_peer_roots": [
        "reader_eof",
        "frontend_control"
      ],
      "staged_type": "SpatialControlStaged",
      "top_level_COMMITTED": "FORBIDDEN"
    },
    "state_layout": {
      "field_group_count": 7,
      "omitted_root_members": [
        "source_flags has no persistent bytes and no canonical group/tag",
        "selection_outer.profile_dimensions is immutable admission metadata; zero canonical bytes and no tag"
      ],
      "owner_state_ref": "#/$defs/ownerState",
      "pointer": "/f03w_owned_fields",
      "previous_schema_amendment_file": "docs/implementation/CALLBACK-OUTPUT-PERSISTENCE-AMENDMENT.md",
      "previous_schema_amendment_id": "selected-adm-installed48-state.v4",
      "profile_binding": "selected EncodingProfile identity plus validated F01E and F03 provider receipts; not mutable root state",
      "profile_dimensions_persistent_state_field": false,
      "profile_dimensions_presence_tag": false,
      "profile_dimensions_root_canonical_bytes": 0,
      "programme_owner_pointer": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json#/properties/owners/properties/spatial_control",
      "schema": "selected-adm-installed48-state.v5",
      "schema_amendment_file": "docs/implementation/SPATIAL-CONTROL-STATE-V5-AMENDMENT.md",
      "schema_file": "parameters/adm-installed48-state-v1.json",
      "source_schema": "selected-adm-installed48-state.v3",
      "source_v3_field_group_count": 8,
      "supplement_manifest_identity": "selected-adm-installed48-state.v5-wash-amendment",
      "supplement_manifest_sha256": "c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38",
      "uses_selected_overlap_projection_as_root": false
    },
    "status": "BLOCKED_ON_SPATIAL_AUDIO_RENDER_AND_PARENT_INTEGRATION",
    "superseding_amendment": {
      "authority": "approved supplement selected-adm-installed48-state.v5-wash-amendment; frozen implementer package remains unchanged",
      "document_bytes": 7315,
      "document_sha256": "a4ea44a5863580fae991d8626ff11e056866614802f331a2352c94b85dc4bede",
      "file": "docs/implementation/SPATIAL-CONTROL-STATE-V5-AMENDMENT.md",
      "input_only_vectors_bytes": 2387,
      "input_only_vectors_file": "INPUT-ONLY-VECTORS.json",
      "input_only_vectors_sha256": "c5292f9a4dfd607704f4a442963b56b3f239e0bc69b8a9d03619fc65bdbc0d69",
      "previous_local_schema_id": "selected-adm-installed48-state.v4",
      "product_execution_scope": "full root canonical serializer, product transaction executor, complete callback/audio replay NOT_RUN",
      "research_basis_files_read": false,
      "schema_file": "SCHEMA-AMENDMENT.json",
      "schema_id": "selected-adm-installed48-state.v5-wash-amendment",
      "schema_sha256": "0bd3be009962e6498c584718e3b63d5aafbdbf423e474c7cf58879385b60fe94",
      "source_schema_id": "selected-adm-installed48-state.v3",
      "spec_file": "SPEC.md",
      "spec_sha256": "c7010d04546b7661663e33ea2822d306665487ff303845bdd541a9ce8a0b4411",
      "supplement_manifest_file": "MANIFEST.json",
      "supplement_manifest_sha256": "c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38"
    },
    "task_id": "P1.3-D-SPATIAL-CONTROL-ROOT-MAP",
    "top_level_committed": "FORBIDDEN",
    "transaction_authority": "same CallbackTransactionId and FrontendCallbackIndex as reader_eof and frontend_control; P1.3-C remains the only eventual commit authority",
    "vector_evidence": {
      "programme_root_pointer": "programme_roots[spatial_control].vector_evidence",
      "scope": "7-callback deterministic synthetic selected profile only; real ADM cold replay is NOT_RUN and package whole implementation readiness is NO"
    }
  }
}
```

## P1.3-D spatial_control root audit

```json
{
  "programme_root_reference": "programme_roots[spatial_control]",
  "task": {
    "blocked_on": [
      "spatial_audio render/audio receipt producer and integration",
      "enclosing CallbackTransaction coordinator"
    ],
    "blockers": [
      "CurrentFeatures math and capability production is locally verified; render/post-window audio production, spatial-audio root integration, and parent CallbackTransaction integration remain deferred",
      "P1.3-C callback implementation waits for all 12 owner roots, including spatial_audio and g>=2 source roots"
    ],
    "callback_local_scratch_not_serialized": [
      "event_control.derived_control38",
      "event_control.derived_regular37",
      "feature.fft4096_work",
      "selection_outer.f03g_gate_output",
      "selection_outer.f03a_selector",
      "selection_outer.f03u_refinement",
      "selection_outer.f03n_nnls",
      "selection_outer.f03s_outer"
    ],
    "canonical_state": "SHA-256 over the complete serialized f03w_owned_fields map in declared order; callback-local scratch and external owner states are excluded",
    "capabilities": [
      "SpatialControlPrepared non-Clone/non-Copy",
      "SpatialControlStaged non-Clone/non-Copy"
    ],
    "clock": "FrontendCallbackIndex main callback clock; source cursor U64 sample position; normal callback count; four cached drains; owner generation increments with the enclosing CallbackTransaction",
    "cold_providers": {
      "f01e_cold": {
        "file": "parameters/f01e-selected-constructor-v1.json",
        "status": "P1.4-A-LOCALLY-VERIFIED"
      },
      "f03_target_cold": {
        "bytes": 3139,
        "file": "parameters/f03-selected-target-cold-v1.json",
        "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
        "prerequisite_status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
        "sha256": "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00"
      }
    },
    "complete_owner_groups": [
      "clocks",
      "source_cache",
      "event_control",
      "render_audio",
      "feature",
      "selection_outer",
      "matrix_assignment"
    ],
    "external_owners_not_embedded": [
      "F05R56 reader_target",
      "F04C component_lfe_receiver_qmf",
      "F05JL joc_after_j05",
      "F13 metadata_queues",
      "F15 source_publication"
    ],
    "implementation_rust": "SourceFlagsPacket and CurrentFeatures math/capability producers are locally verified; full P1.3-D remains blocked on spatial_audio render integration and parent CallbackTransaction integration; D1 status is unchanged",
    "local_commit": "FORBIDDEN",
    "mapping_status": "MAPPING_COMPLETE_BLOCKED",
    "owner_receipt_order_and_wrapper": "P1.3-C approves 12-owner ASCII order and schema v2 with 13 fields; wrapper has no digest/HMAC; implementation waits for root availability",
    "packet_producer_blockers": [],
    "prerequisites": [
      {
        "status": "CALLBACK_CONTRACT_APPROVED_WAITING_OWNER_ROOTS",
        "task_id": "P1.3-C-CALLBACK-TRANSACTION-CONTRACT-MAP"
      },
      {
        "status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
        "task_id": "P1.4-B-F03-SELECTED-TARGET-COLD-PROVIDER-CONTRACT"
      }
    ],
    "previous_local_amendment": {
      "document_bytes": 4366,
      "document_sha256": "2e55a2e142b6482e456e40a9f3154f82574c2ed4259acab5ebab08369a84d474",
      "file": "docs/implementation/CALLBACK-OUTPUT-PERSISTENCE-AMENDMENT.md",
      "schema_id": "selected-adm-installed48-state.v4"
    },
    "programme_owner_schema": {
      "file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "owner_state_fields": [
        "owner_id",
        "generation",
        "state_sha256",
        "cold_sha256",
        "state_ref"
      ],
      "pointer": "#/properties/owners/properties/spatial_control",
      "ref": "#/$defs/ownerState"
    },
    "programme_root": "spatial_control",
    "provider_prerequisite": {
      "identity_source": "approved package MANIFEST.json entry",
      "required_provider": "parameters/f03-selected-target-cold-v1.json",
      "status": "SATISFIED_READY_FOR_STAGING_IMPLEMENTATION",
      "task_id": "P1.4-B-F03-SELECTED-TARGET-COLD-PROVIDER-CONTRACT"
    },
    "receipt_order": "12-owner ASCII order approved by P1.3-C; v2 wrapper has no digest/HMAC and stable JSON is test-only",
    "root_audit": "source layout selected-adm-installed48-state.v3 remains bound to the approved package; local v4 and approved v5 amendments define the logical persistent root; callback outputs and immutable profile_dimensions are excluded from mutable state and seven persistent groups remain",
    "stage_authority": "F03W emits SpatialControlReceipt after all F03W leaf stages and downstream receipts pass",
    "stage_only": {
      "join_metadata": [
        "CallbackTransactionId",
        "FrontendCallbackIndex",
        "before_generation",
        "before_sha256",
        "after_sha256",
        "F01E provider lineage",
        "F03 target provider lineage",
        "external owner receipts"
      ],
      "linear": true,
      "local_commit": "FORBIDDEN",
      "prepared_type": "SpatialControlPrepared",
      "required_peer_roots": [
        "reader_eof",
        "frontend_control"
      ],
      "staged_type": "SpatialControlStaged",
      "top_level_COMMITTED": "FORBIDDEN"
    },
    "state_layout": {
      "field_group_count": 7,
      "omitted_root_members": [
        "source_flags has no persistent bytes and no canonical group/tag",
        "selection_outer.profile_dimensions is immutable admission metadata; zero canonical bytes and no tag"
      ],
      "owner_state_ref": "#/$defs/ownerState",
      "pointer": "/f03w_owned_fields",
      "previous_schema_amendment_file": "docs/implementation/CALLBACK-OUTPUT-PERSISTENCE-AMENDMENT.md",
      "previous_schema_amendment_id": "selected-adm-installed48-state.v4",
      "profile_binding": "selected EncodingProfile identity plus validated F01E and F03 provider receipts; not mutable root state",
      "profile_dimensions_persistent_state_field": false,
      "profile_dimensions_presence_tag": false,
      "profile_dimensions_root_canonical_bytes": 0,
      "programme_owner_pointer": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json#/properties/owners/properties/spatial_control",
      "schema": "selected-adm-installed48-state.v5",
      "schema_amendment_file": "docs/implementation/SPATIAL-CONTROL-STATE-V5-AMENDMENT.md",
      "schema_file": "parameters/adm-installed48-state-v1.json",
      "source_schema": "selected-adm-installed48-state.v3",
      "source_v3_field_group_count": 8,
      "supplement_manifest_identity": "selected-adm-installed48-state.v5-wash-amendment",
      "supplement_manifest_sha256": "c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38",
      "uses_selected_overlap_projection_as_root": false
    },
    "status": "BLOCKED_ON_SPATIAL_AUDIO_RENDER_AND_PARENT_INTEGRATION",
    "superseding_amendment": {
      "authority": "approved supplement selected-adm-installed48-state.v5-wash-amendment; frozen implementer package remains unchanged",
      "document_bytes": 7315,
      "document_sha256": "a4ea44a5863580fae991d8626ff11e056866614802f331a2352c94b85dc4bede",
      "file": "docs/implementation/SPATIAL-CONTROL-STATE-V5-AMENDMENT.md",
      "input_only_vectors_bytes": 2387,
      "input_only_vectors_file": "INPUT-ONLY-VECTORS.json",
      "input_only_vectors_sha256": "c5292f9a4dfd607704f4a442963b56b3f239e0bc69b8a9d03619fc65bdbc0d69",
      "previous_local_schema_id": "selected-adm-installed48-state.v4",
      "product_execution_scope": "full root canonical serializer, product transaction executor, complete callback/audio replay NOT_RUN",
      "research_basis_files_read": false,
      "schema_file": "SCHEMA-AMENDMENT.json",
      "schema_id": "selected-adm-installed48-state.v5-wash-amendment",
      "schema_sha256": "0bd3be009962e6498c584718e3b63d5aafbdbf423e474c7cf58879385b60fe94",
      "source_schema_id": "selected-adm-installed48-state.v3",
      "spec_file": "SPEC.md",
      "spec_sha256": "c7010d04546b7661663e33ea2822d306665487ff303845bdd541a9ce8a0b4411",
      "supplement_manifest_file": "MANIFEST.json",
      "supplement_manifest_sha256": "c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38"
    },
    "task_id": "P1.3-D-SPATIAL-CONTROL-ROOT-MAP",
    "top_level_committed": "FORBIDDEN",
    "transaction_authority": "same CallbackTransactionId and FrontendCallbackIndex as reader_eof and frontend_control; P1.3-C remains the only eventual commit authority",
    "vector_evidence": {
      "programme_root_pointer": "programme_roots[spatial_control].vector_evidence",
      "scope": "7-callback deterministic synthetic selected profile only; real ADM cold replay is NOT_RUN and package whole implementation readiness is NO"
    }
  }
}
```

## P1.3-E spatial_audio programme-root mapping

```json
{
  "programme_root": {
    "callback_local_only_fields": [
      "prefix row accumulators and prefix/regular output rows",
      "matrix[g], destinations[g], current_lfe[g], installed48[g] input views",
      "Type10[g], final_six[g], target/carrier QMF output windows and all per-callback receipts"
    ],
    "canonical_state": {
      "byte_level_encoding_complete": true,
      "callback_local_scratch": "excluded",
      "cold_preimage_reconstructed": true,
      "cold_state_sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e",
      "digest": "SHA-256 over complete F04CompositionState canonical bytes; fresh typed construction and project encoder output match the complete post-construction KAT byte-for-byte",
      "encoding": "F04_STATE_NODE_V1: N/B+U64LE/I+I64LE/F+F32LE/Z+U8/T+U64LE/D+class+declared fields",
      "encoding_id": "F04_STATE_NODE_V1",
      "expected_cold_bytes": 813733,
      "field_order": [
        "callback_index",
        "retired",
        "component_state",
        "component_parameter_state",
        "component_filter_history",
        "lfe_state",
        "receiver_state",
        "fitting_qmf_state",
        "receiver_callback_count"
      ],
      "local_kat_bytes_match": true,
      "schema_semantic_sha256_is_state_digest": false
    },
    "clock": {
      "callback_index_type": "FrontendCallbackIndex",
      "generation_rule": "GTC owner generation increments once only when the enclosing CALLBACK commits",
      "join": "must equal the parent CallbackTransaction FrontendCallbackIndex; it is not an independent clock",
      "receiver_count_type": "ReceiverCallbackCount",
      "receiver_rule": "g<2 advances component/LFE only; g>=2 maps r=g-2 and advances receiver/QMF once"
    },
    "cold_provider": {
      "complete_constructor_status": "LOCALLY_VERIFIED_TYPED_COLD_CONSTRUCTION",
      "generation_specs": [
        "specs/04-whole-spatial-chain-composition.md",
        "specs/04-component-parameter-state.md",
        "specs/04-receiver-state-and-routing.md",
        "specs/04-qmf-receiver-whole-binding.md"
      ],
      "kind": "GENERATED_PROVIDER",
      "null_sentinels_are_preserved": true,
      "provider_files": [],
      "schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
      "unknown_cold_arrays_zero_filled": false
    },
    "cold_sentinels": {
      "lfe_input_frame_indexes": "3 x None",
      "lfe_rows": "3 x None",
      "receiver_target_coefficients": "None",
      "reserved_receiver_temporal_slot": "temporal state index 4 remains cold and unconsumed",
      "zero_fill_policy": "Only arrays explicitly specified positive zero in the pinned constructor may be zero; None sentinels remain None; missing arrays block implementation"
    },
    "cold_state_resolution": {
      "expected_cold_sha256_is_preimage": false,
      "f04_state_node_encoding": {
        "constraints": [
          "bool is classified as Z before integer I",
          "I is signed I64",
          "B remains opaque raw bytes and is not promoted to F or I",
          "tuple/list share T framing",
          "class and field declarations must match the approved typed schema",
          "no dict/set/string/reference/cycle/alignment/terminator/header/checksum/digest"
        ],
        "dataclass": "D, U16LE class-name byte length, ASCII class name, U16LE field count, then each declaration-ordered field's U16LE ASCII name length/name and recursively encoded value; no D body length",
        "f32": "R32 conversion; reject non-finite and overflow; preserve sign bit generally; the cold-state constructor rejects negative zero",
        "status": "RESOLVED_BY_F04_COLD_SUPPLEMENT",
        "tags": {
          "B": "U64LE byte length then raw bytes",
          "D": "ASCII class and declaration-ordered ASCII fields with U16LE lengths",
          "F": "finite F32LE after R32",
          "I": "signed I64LE",
          "N": "None; tag only",
          "T": "U64LE item count then recursive items",
          "Z": "U8 0 or 1"
        }
      },
      "local_kat_byte_comparison": "PASS: 813733 actual project-encoder bytes equal the post-construction KAT",
      "local_preimage_reconstructed": true,
      "status": "RESOLVED_BY_F04_COLD_SUPPLEMENT",
      "supplement": {
        "expected_cold_bytes": 813733,
        "expected_cold_sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e",
        "files": [
          {
            "bytes": 274680,
            "file": "COLD-CONSTRUCTOR-RECIPE.json",
            "sha256": "4bc96ea5f2908a1c911a6b57e047be9cd60589b0ca29aa2f457cfd154fa0632a"
          },
          {
            "bytes": 813733,
            "file": "F04-COLD-PREIMAGE.bin",
            "sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e"
          },
          {
            "bytes": 7110,
            "file": "INPUT-ONLY-VECTORS.json",
            "sha256": "f42e538bb4b622dbe64660f79df7f5ec04e3a84599be9141c76b06940de4fcd5"
          },
          {
            "bytes": 5014,
            "file": "MUTATION-AND-TEMPORAL-VECTORS.json",
            "sha256": "d207abd2bd86cefdad375b6dd04078f5bdacb6d96824cc331884054ddc30a47e"
          },
          {
            "bytes": 4364,
            "file": "PREIMAGE-LAYOUT.json",
            "sha256": "2eb372516cd24feb053c0aaf847cabdc1a924e02df58f1a51c1cd077e5a8da07"
          },
          {
            "bytes": 12866,
            "file": "SPEC.md",
            "sha256": "2b23840a75bcb04e8da19499d0b0ad4ed688e5041a49e1f138f9ddd0c898785a"
          },
          {
            "bytes": 29347,
            "file": "TEMPORAL-COLD-0.bin",
            "sha256": "9246ae1f51e4ecd21b681748f08152276ac4c545961e7c7834c9b087a8707e8b"
          },
          {
            "bytes": 2677,
            "file": "TEMPORAL-COLD-1.bin",
            "sha256": "1cb9a2ccd0c4f24e32d93dd075bf1e75a84e789e53336dd9f7164a116fc4b1d1"
          },
          {
            "bytes": 7321,
            "file": "TEMPORAL-COLD-2.bin",
            "sha256": "e6bd6c37dd859a91814b555936b25d230bd776e01867b3d9311dd5c4750f782b"
          },
          {
            "bytes": 12454,
            "file": "TEMPORAL-COLD-3.bin",
            "sha256": "42e77b078aa82202c9513b0d450b5559ab03b95e145f31cc31493e717793c77e"
          },
          {
            "bytes": 107767,
            "file": "TEMPORAL-COLD-4.bin",
            "sha256": "da2284560f5f4567406a95532f87a077a568d0bf67974ebdab5989e6efb36c89"
          },
          {
            "bytes": 107767,
            "file": "TEMPORAL-COLD-5.bin",
            "sha256": "da2284560f5f4567406a95532f87a077a568d0bf67974ebdab5989e6efb36c89"
          }
        ],
        "identity": "f04-spatial-audio-cold-state.v1",
        "kat_usage": "identity hashed opaquely; state comparison only after fresh typed construction and project encoder output",
        "manifest_sha256": "77ad0562fcd4efa1e8fe37cc1dc7219570b49b9b332cca1f1273ccda2e2cef9c",
        "research_basis_files_read": false,
        "verified_file_count": 12
      },
      "temporal_fields": {
        "buffers": "lane-major tuple; each lane is an opaque B value containing 2*span little-endian F32 positive-zero words",
        "independent_same_shape_nodes": "receiver_state.temporal_states[4] and fitting_qmf_state.target_temporal own separate buffers despite equal cold contents",
        "index_semantics": "index_28=read=span; index_32=write=0",
        "nodes": [
          {
            "bytes_per_lane": 9736,
            "f32_per_lane": 2434,
            "lanes": 3,
            "offset": 226178,
            "path": "receiver_state.temporal_states[0]",
            "sha256": "9246ae1f51e4ecd21b681748f08152276ac4c545961e7c7834c9b087a8707e8b",
            "span": 1217,
            "wire_bytes": 29347
          },
          {
            "bytes_per_lane": 504,
            "f32_per_lane": 126,
            "lanes": 5,
            "offset": 255525,
            "path": "receiver_state.temporal_states[1]",
            "sha256": "1cb9a2ccd0c4f24e32d93dd075bf1e75a84e789e53336dd9f7164a116fc4b1d1",
            "span": 63,
            "wire_bytes": 2677
          },
          {
            "bytes_per_lane": 7200,
            "f32_per_lane": 1800,
            "lanes": 1,
            "offset": 258202,
            "path": "receiver_state.temporal_states[2]",
            "sha256": "e6bd6c37dd859a91814b555936b25d230bd776e01867b3d9311dd5c4750f782b",
            "span": 900,
            "wire_bytes": 7321
          },
          {
            "bytes_per_lane": 2048,
            "f32_per_lane": 512,
            "lanes": 6,
            "offset": 265523,
            "path": "receiver_state.temporal_states[3]",
            "sha256": "42e77b078aa82202c9513b0d450b5559ab03b95e145f31cc31493e717793c77e",
            "span": 256,
            "wire_bytes": 12454
          },
          {
            "bytes_per_lane": 7168,
            "f32_per_lane": 1792,
            "lanes": 15,
            "offset": 277977,
            "path": "receiver_state.temporal_states[4]",
            "sha256": "da2284560f5f4567406a95532f87a077a568d0bf67974ebdab5989e6efb36c89",
            "span": 896,
            "wire_bytes": 107767
          },
          {
            "bytes_per_lane": 7168,
            "f32_per_lane": 1792,
            "lanes": 15,
            "offset": 392696,
            "path": "fitting_qmf_state.target_temporal",
            "sha256": "da2284560f5f4567406a95532f87a077a568d0bf67974ebdab5989e6efb36c89",
            "span": 896,
            "wire_bytes": 107767
          }
        ],
        "status": "RESOLVED_BY_F04_COLD_SUPPLEMENT",
        "stride_bytes": 4
      }
    },
    "commit_authority": "enclosing GTC CallbackTransactionCoordinator only; spatial_audio has no independent commit or top-level COMMITTED authority",
    "eof_drain": "process each actual parent callback including four cached drains; component/LFE advance every callback, receiver/QMF only for g>=2; no extra QMF zero flush or synthetic callback",
    "excluded_provider_bindings": [
      {
        "bytes": 2783,
        "file": "parameters/spread-authorized-v1.json",
        "kind": "FILE",
        "owner": "spatial_control/F03W installed48 producer path; not a spatial_audio persistent field/provider",
        "sha256": "5611b37f2a997f75da0ba920e394c0e36eddd9477e11ba94d44852d3e12fd0c9"
      },
      {
        "bytes": 1467,
        "file": "parameters/diffusion-mathematical-v1.json",
        "kind": "GENERATED",
        "owner": "spatial_control/F03W; selected D11-DP production config is not supplied by this route",
        "sha256": "d690512cd204e97fc04a35c03519ed3005e26cb57408432fe30bfde63c2319aa"
      },
      {
        "bytes": 1770,
        "file": "parameters/diffusion-constructor-mathematical-v1.json",
        "kind": "GENERATED",
        "owner": "spatial_control/F03W; not part of F04CompositionState",
        "sha256": "7a710d68b00c14d6b9b3f9fa3313579c5739ccce408655ea3252ad1ee4acd106"
      }
    ],
    "fatal_behavior": "F04_STATE_ERROR, F04_PIPELINE_RETIRED, F04_CALLBACK_ORDER, F04_INPUT_ERROR, F04_PROFILE_UNSUPPORTED, or F04_NUMERIC_FAILURE yields no normal output or committed after-state; parent callback abort preserves all owner before snapshots and retires the programme",
    "fields": [
      {
        "cold": "0",
        "name": "callback_index",
        "runtime_clock_type": "FrontendCallbackIndex",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/0",
        "type": "I64LE"
      },
      {
        "authority": "F04-local fail-stop guard only; GTC programme status and fatal authority remain with CallbackTransactionCoordinator",
        "cold": "false",
        "name": "retired",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/1",
        "type": "U8 boolean"
      },
      {
        "cold": "overlap_history_rows=F32[15][1536] positive zero; source_ring_rows=F32[15][1472] positive zero",
        "name": "component_state",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/2",
        "type": "ComponentAudioState{overlap_history_rows:T15<B6144>,source_ring_rows:T15<B5888>}"
      },
      {
        "cold": "peak=+0; previous_gain=1; hold=0; envelope=1; smoothed_gain=1; target_gain=1; output_level=2^-8; blocks_processed=0",
        "name": "component_parameter_state",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/3",
        "type": "ComponentFilterParameterState{peak:F32,previous_gain:F32,hold:I64LE,envelope:F32,smoothed_gain:F32,target_gain:F32,output_level:F32,blocks_processed:I64LE}"
      },
      {
        "cold": "F32[15][64] positive zero",
        "name": "component_filter_history",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/4",
        "type": "T15<B256>"
      },
      {
        "cold": "write_index=0; read_index=1; rows=[None,None,None]; input_frame_indexes=[None,None,None]; model_samples_consumed=0",
        "name": "lfe_state",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/5",
        "type": "LfeDelayRingState{write_index:I64LE,read_index:I64LE,rows:T3<None>,input_frame_indexes:T3<None>,model_samples_consumed:I64LE}"
      },
      {
        "cold": "All sample/history arrays use the specified cold constructors; RingState indices are write=224/read=0; six independent TemporalState nodes use stride_bytes=4, index_28=span/read, index_32=0/write, and lane-major 2*span F32 positive-zero buffers; descriptor cold fields are documented; target_coefficients=None; temporal slot 4 remains reserved and unconsumed",
        "name": "receiver_state",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/6",
        "type": "ModelState; exact recursive layout is pinned by the state schema pointer and F04RB.7"
      },
      {
        "cold": "target/history/window/delay/FIR arrays are their declared positive-zero shapes; target_temporal is an independent span=896 TemporalState with stride_bytes=4, index_28=896/read, index_32=0/write, and lane-major 2*span F32 positive-zero buffers; mode=0 fitting histories remain distinct from receiver mode=1 histories",
        "name": "fitting_qmf_state",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/7",
        "type": "QmfWindowState; target_temporal, target_history, previous_targets, previous_carriers, fitting_delay, fitting_fir"
      },
      {
        "cold": "0; increments only when Type10 exists (g>=2)",
        "name": "receiver_callback_count",
        "runtime_clock_type": "ReceiverCallbackCount",
        "schema_pointer": "#/whole_state_by_boundary/cold/fields/8",
        "type": "I64LE"
      }
    ],
    "implementation_status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__DSP_PRODUCERS_DEFERRED",
    "linear_capabilities": {
      "machine": "SpatialAudioMachine; one committed root owner; non-Clone and non-Copy",
      "prepared": "SpatialAudioPrepared; non-Clone/non-Copy; consumed by stage or discard",
      "staged": "SpatialAudioStaged; non-Clone/non-Copy; consumed by enclosing coordinator commit or abort"
    },
    "materialization_status": "MAPPING_COMPLETE_BLOCKED",
    "ownership_partition": {
      "joc_persistent_fields": [
        "F05JL owns temporal, geometry, fit, retry, Huffman, and ID14 writer state; F05JL consumes the F04C receipt and does not own/recompute F04R/F04RB histories"
      ],
      "other_owner_fields": [
        "F05R56 reader/target histories",
        "F13 metadata FIFO/timer/EMDF assembly state",
        "F15 source publication and durable output state"
      ],
      "spatial_audio_persistent_fields": [
        "component overlap/ring/filter histories and component parameter state",
        "direct-LFE ring and source-generation slots",
        "receiver ModelState including receiver audio mode=1 QMF histories",
        "QmfWindowState including distinct mode=0 fitting delay/FIR and target/carrier windows"
      ],
      "spatial_control_persistent_fields": [
        "raw/previous PCM and previous-current48 histories",
        "installed48 producer histories",
        "diffusion instances and previous diffusion outputs",
        "targets, assignment, markers, and matrix state"
      ]
    },
    "post_window_fanout": {
      "boundary": "INSTALLED48_POST_WINDOW_48X3072",
      "construction": "future spatial_control/F03W renderer only through a private module boundary; no public raw/default constructor",
      "consumers": {
        "current_features": "consumes CurrentFeaturesAudioView and an explicit expected CallbackPhase",
        "future_spatial_audio_assembler": "consumes SpatialAudioPostWindowView with same-callback matrix/destination/LFE inputs; assembler not implemented"
      },
      "fanout": {
        "operation": "fanout(self)",
        "payload_allocation": "one shared immutable Arc allocation; fanout clones only the Arc handle and never copies samples",
        "proposal_and_views_clone_copy": false,
        "returns_exactly": [
          "CurrentFeaturesAudioView",
          "SpatialAudioPostWindowView"
        ]
      },
      "fanout_status": "LOCALLY_VERIFIED_ONE_SHOT_TWO_VIEW_CAPABILITY__RENDER_PRODUCER_DEFERRED",
      "integration_status": "fanout capability locally verified; render producer, matrix/destination producer, F04 assembler, and parent transaction remain deferred",
      "interface": "PostWindowRenderProposal",
      "lineage": [
        "programme and attempt",
        "CallbackTransactionId, FrontendCallbackIndex, and explicit CallbackPhase NORMAL or CACHED_DRAIN",
        "selected profile and F01E/F03 provider lineage",
        "spatial_control before generation and complete root digest",
        "typed render-stage identity and spatial-control after-proposal identity",
        "payload byte count and SHA-256 of exact source-major F32 U32LE bytes"
      ],
      "payload": "one immutable Arc<[FiniteF32]> of exactly 48x3072 source-major samples",
      "producer_owner": "spatial_control/F03W render producer",
      "producer_stage": "same-callback W3072/post-window installed48 boundary before F04C consumer processing"
    },
    "programme_owner_binding": {
      "binding_semantics": "The generic GTC OwnerState points to the complete F04CompositionState cold-schema candidate. The published cold digest is retained as an expected identity only; exact temporal cold scalars and the full F04 D-node byte grammar are not uniquely bound, so the canonical cold preimage and GTC cold_sha256 binding are not yet verified.",
      "cold_sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e",
      "file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
      "pointer": "#/properties/owners/properties/spatial_audio",
      "ref": "#/$defs/ownerState",
      "state_ref_value": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json#/whole_state_by_boundary/cold",
      "state_schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
      "state_schema_pointer": "#/whole_state_by_boundary/cold",
      "state_schema_sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c",
      "state_schema_type": "F04CompositionState",
      "state_semantic_sha256": "e1ef560564678a04494a9a563b1e431582fd0a48c8524ff4e4b47cf7a880655c",
      "uses_f04c_local_receipt_as_root": false,
      "uses_selected_overlap_projection_as_root": false,
      "wrapper_fields": [
        "owner_id",
        "generation",
        "state_sha256",
        "cold_sha256",
        "state_ref"
      ]
    },
    "readiness": {
      "blockers": [
        "P1.4-D1/D2 locally verify all five F04 parameter providers; remaining F04 DSP, render production, and parent integration are deferred",
        "PostWindow fanout capability is locally verified; F03W render producer, matrix/destination producer, F04 assembler, and parent CallbackTransaction remain deferred",
        "P1.3-C remains waiting for the other owner roots and g>=2 source-stage roots; no parent CallbackTransaction implementation exists"
      ],
      "implementation_status": "BLOCKED_ON_F04_DSP_RENDER_PARENT",
      "mapping_status": "MAPPING_COMPLETE_BLOCKED"
    },
    "receipt": {
      "fields": [
        "owner_id",
        "before_generation",
        "before_sha256",
        "after_generation",
        "after_sha256",
        "changed",
        "commit_count",
        "transaction_id"
      ],
      "generation": "cold=0; enclosing successful CALLBACK commit increments exactly once; failed/rejected callback leaves before values unchanged",
      "schema_file": "test-vectors/GLOBAL-TRANSACTION-PROGRAMME-STATE-SCHEMA.json",
      "schema_pointer": "#/$defs/ownerReceipt",
      "transaction_id": "same CallbackTransactionId as every other owner snapshot",
      "type": "OwnerReceipt<SpatialAudio>"
    },
    "reset_authority": "programme-origin full restart; reconstruct the complete cold state and discard every staged token; no resume within RETIRED attempt",
    "root_id": "spatial_audio",
    "runtime_providers": [
      {
        "bytes": 2704,
        "file": "parameters/window-q15.json",
        "id": "F04_WINDOW_Q15",
        "kind": "GENERATED",
        "p1_4_contract": "P1.4-D1/D2 LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS; DSP consumption NOT_YET_VALIDATED",
        "permission": "ordinary generated non-secret parameters; private vendor key absent",
        "sha256": "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130",
        "shape": "receiver_blend32 Q15 and component_kernel64 Q15; formula-generated F32LE views are identity-pinned"
      },
      {
        "bytes": 1077,
        "file": "parameters/component-parameter-mathematical-v1.json",
        "id": "F04_COMPONENT_PARAMETER_MATH_V1",
        "kind": "GENERATED",
        "p1_4_contract": "P1.4-D1/D2 LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS; DSP consumption NOT_YET_VALIDATED",
        "permission": "internal mathematical generation; implementation_ready=NO; no source table copied",
        "sha256": "5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5",
        "shape": "8 Q15 integers and 8 F32 coefficient/pivot/weight words at 48 kHz and 64 samples per block"
      },
      {
        "bytes": 20325,
        "file": "parameters/qmf-receiver-functional-v1.json",
        "id": "F04_QMF_RECEIVER_FUNCTIONAL_V1",
        "kind": "FILE",
        "p1_4_contract": "P1.4-D1/D2 LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS; DSP consumption NOT_YET_VALIDATED",
        "permission": "authorized internal isolated implementation only; no secret, source, raw table, compiled image, or address",
        "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c",
        "shape": "829 array F32 words plus 12 scalar F32 words; QWIN synthesis 640, selected band0 real/imag 21 each, LFE postfilter 20, descriptor thresholds 127"
      },
      {
        "bytes": 6983,
        "file": "parameters/qmf-mathematical-fft-v1.json",
        "id": "F04_QMF_MATH_V1",
        "kind": "GENERATED",
        "p1_4_contract": "P1.4-D1/D2 LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS; DSP consumption NOT_YET_VALIDATED",
        "permission": "generated mathematical values; compiled values not used",
        "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431",
        "shape": "radix-2 stage roots and rotations for QMF_FFT_RADIX2_MATH_V1"
      },
      {
        "bytes": 4861,
        "file": "parameters/descriptor-mathematical.json",
        "id": "F04_DESCRIPTOR_MATH_V1",
        "kind": "GENERATED",
        "p1_4_contract": "P1.4-D1/D2 LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS; DSP consumption NOT_YET_VALIDATED",
        "permission": "mathematical generation; audio acceptance pending",
        "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197",
        "shape": "128 gain words and 127 threshold words"
      }
    ],
    "stage_authority": {
      "owner": "F04C",
      "receipt": "selected-j05-owner-receipt.v1 is a local immutable stage receipt, not the programme root or GTC owner receipt",
      "stage": "component, direct-LFE, receiver and both QMF state transitions run once; emit PRIMING for g<2 and selected-j05-owner-receipt.v1 for g>=2"
    }
  },
  "task": {
    "blocker_file": "docs/implementation/SPATIAL-AUDIO-COLD-STAGING-BLOCKER.md",
    "blockers": [
      "P1.3-E1 typed cold construction, byte-level KAT comparison, and staging foundation are locally verified; F04 DSP remains deferred",
      "P1.4-D1/D2 locally validate all five registered F04 providers; remaining F04 DSP, PostWindow renderer, matrix/destination producer, F04 assembler, and parent CallbackTransaction remain deferred"
    ],
    "cold_preimage_reconstructed": true,
    "cold_sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e",
    "implementation_status": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
    "mapping_status": "MAPPING_COMPLETE_BLOCKED",
    "p1_3_c_prerequisite": "P1.3-C remains CALLBACK_CONTRACT_APPROVED_WAITING_OWNER_ROOTS until the remaining owner roots and g>=2 source-stage roots are materialized",
    "p1_4_prerequisite": "P1.4-D1 and P1.4-D2 locally verify all five registered F04 parameter providers; remaining F04 DSP stays unimplemented",
    "post_window_packet_prerequisite": "typed one-shot two-view fanout capability is locally verified; F03W renderer, matrix/destination producer, F04 assembler, and parent callback transaction remain deferred",
    "programme_root": "spatial_audio",
    "root_field_count": 9,
    "root_schema_pointer": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json#/whole_state_by_boundary/cold",
    "root_state_type": "F04CompositionState",
    "scope": "Map the spatial_audio GTC owner candidate and its cold/provider prerequisites; F04 cold serialization is specified by the verified supplement; no DSP or Rust state implementation",
    "status": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
    "task_id": "P1.3-E-SPATIAL-AUDIO-PROGRAMME-ROOT-MAP"
  }
}
```

## P1.3-E1 spatial_audio cold-state/staging foundation

```json
{
  "blocked_on": [],
  "blocker_file": "docs/implementation/SPATIAL-AUDIO-COLD-STAGING-BLOCKER.md",
  "cold_spec_status": "RESOLVED_BY_F04_COLD_SUPPLEMENT",
  "cold_state_actual_bytes": 813733,
  "cold_state_actual_sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e",
  "expected_cold_bytes": 813733,
  "expected_cold_sha256": "35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e",
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/mod.rs",
    "crates/jocforge-domain/src/timeline/index.rs",
    "crates/jocforge-domain/src/timeline/mod.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 35171,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "6e3154571999492fdf2f881147ee9bb1a111eb191bd9cba9aaace91075840e03"
    },
    {
      "bytes": 2174,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "697124fac06aa7243049d258ede0634d3b5de721743b1238c049ff2b13752afd"
    },
    {
      "bytes": 2290,
      "path": "crates/jocforge-domain/src/timeline/index.rs",
      "sha256": "911be2413816ebe1fde51ceda0066e9c088f92ae7c8b045cee49aac37540ec7d"
    },
    {
      "bytes": 586,
      "path": "crates/jocforge-domain/src/timeline/mod.rs",
      "sha256": "7f2099065d657bdacab116d921a63107a180930086b73a42557881aa3acdb7b2"
    }
  ],
  "implementation_prohibitions": [
    "do not read F04-COLD-PREIMAGE.bin as state input or return its bytes instead of encoder output",
    "do not implement any DSP, F04 provider validator, PostWindow renderer/fanout, or local commit/abort/fatal authority",
    "do not merge the two span=896 temporal nodes or zero-fill LFE None sentinels"
  ],
  "implementation_status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__DSP_PRODUCERS_DEFERRED",
  "mapping_status": "MAPPING_COMPLETE",
  "programme_root": "spatial_audio",
  "scope": "Construct and serialize the complete F04CompositionState cold root with a bounded typed F04_STATE_NODE_V1 encoder, then provide a non-Clone storage/staging foundation; do not implement DSP, provider validation, PostWindow render/fanout, or local commit/abort/fatal",
  "status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__DSP_PRODUCERS_DEFERRED",
  "supplement_manifest_sha256": "77ad0562fcd4efa1e8fe37cc1dc7219570b49b9b332cca1f1273ccda2e2cef9c",
  "task_id": "P1.3-E1-SPATIAL-AUDIO-COLD-STAGING-FOUNDATION",
  "test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/tests.rs",
    "crates/jocforge-domain/tests/spatial_audio_cold_staging.rs"
  ],
  "test_inventory": [
    {
      "bytes": 6848,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/tests.rs",
      "sha256": "2cea166d6103a544200268ab1560b53ceec10fc5a190afcb7bf9e0837478186b"
    },
    {
      "bytes": 9923,
      "path": "crates/jocforge-domain/tests/spatial_audio_cold_staging.rs",
      "sha256": "0db769e058314f49f35266af6fdcff9c7cf9ba58e126627711a21a722d3a5fa0"
    }
  ],
  "test_vector_fixture_files": [
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/F04-COLD-PREIMAGE.bin",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/INPUT-ONLY-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/MUTATION-AND-TEMPORAL-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/TEMPORAL-COLD-0.bin",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/TEMPORAL-COLD-1.bin",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/TEMPORAL-COLD-2.bin",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/TEMPORAL-COLD-3.bin",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/TEMPORAL-COLD-4.bin",
    "crates/jocforge-domain/tests/fixtures/spatial_audio_cold/TEMPORAL-COLD-5.bin"
  ],
  "verification": {
    "local_commit_abort_fatal": "NOT_IMPLEMENTED",
    "mutation_vector_case_count": 15,
    "primitive_input_case_count": 12,
    "root_kat_byte_comparison": "PASS",
    "temporal_kat_count": 6
  }
}
```

## P1.3-D1 spatial_control staging-only implementation

```json
{
  "authority": {
    "abort_fatal": "FORBIDDEN",
    "local_commit": "FORBIDDEN",
    "noncold_prepare": "crate-private and deferred until a future producer can supply the complete typed after proposal; no test-only constructor",
    "prepare": "pure validation against shared CallbackTransactionId, FrontendCallbackIndex, before_generation, before_sha256, selected EncodingProfile identity and both provider receipt lineages",
    "stage": "produce a complete immutable after-state and recomputed full v5 root digest without mutating committed state; equal state hashes do not authorize cross-profile/provider tokens",
    "top_level_COMMITTED": "FORBIDDEN"
  },
  "callback_output_packets": {
    "capability": "opaque, producer-owned, linear",
    "cold_reset_read": "NOT_READY",
    "forged_lineage": "FORBIDDEN",
    "names": [
      "SourceFlagsPacket",
      "CurrentFeaturesPacket"
    ],
    "parent_transaction": "future producer receipt must bind the packet to the same CallbackTransactionId and FrontendCallbackIndex as the complete typed after proposal",
    "producer_status": "FUTURE_PRODUCER_DEFERRED",
    "public_raw_array_constructor": "FORBIDDEN",
    "root_canonical_bytes": 0,
    "root_member": false,
    "zero_or_default_packet": "FORBIDDEN"
  },
  "capabilities": {
    "SpatialControlMachine": "single owner, non-Clone/non-Copy, immutable committed access",
    "SpatialControlPrepared": "linear, non-Clone/non-Copy",
    "SpatialControlStaged": "linear, non-Clone/non-Copy"
  },
  "cold_inputs": [
    "ValidatedF01EColdProvider",
    "ValidatedF03TargetColdProvider",
    "selected EncodingProfile identity",
    "only the GENERATED cold rules recorded in the v5 mapping"
  ],
  "deferred_case_resolutions": [
    {
      "evidence_scope": "20 synthetic vectors plus one authorized first-callback boundary literal; this is not D1 evidence",
      "id": "flags_before_history",
      "resolved_by": "SOURCE_FLAGS_PRODUCER",
      "status": "LOCALLY_VERIFIED_NUMERIC_AND_PACKET"
    }
  ],
  "implementation": {
    "clock_domain_file": "crates/jocforge-domain/src/timeline/index.rs",
    "file_inventory": [
      {
        "bytes": 26347,
        "path": "crates/jocforge-domain/src/spatial/spatial_control.rs",
        "sha256": "3c13f67f84698b32d568bf9fda223878eb3925a95c7fcbe30e0d0ef2f1b8fcd2"
      },
      {
        "bytes": 725,
        "path": "crates/jocforge-domain/src/transaction/mod.rs",
        "sha256": "3ca14198acbe03b7dcb431319a63004b1ef68bae80396df930e007d576d85e82"
      },
      {
        "bytes": 2290,
        "path": "crates/jocforge-domain/src/timeline/index.rs",
        "sha256": "911be2413816ebe1fde51ceda0066e9c088f92ae7c8b045cee49aac37540ec7d"
      },
      {
        "bytes": 2174,
        "path": "crates/jocforge-domain/src/spatial/mod.rs",
        "sha256": "697124fac06aa7243049d258ede0634d3b5de721743b1238c049ff2b13752afd"
      },
      {
        "bytes": 9428,
        "path": "crates/jocforge-domain/tests/spatial_control_foundation.rs",
        "sha256": "da7854c7a5ed2a1651114214d71ca266a3e41fc20ccba1958c74ee62f4264ae1"
      }
    ],
    "module_export_file": "crates/jocforge-domain/src/spatial/mod.rs",
    "module_file": "crates/jocforge-domain/src/spatial/spatial_control.rs",
    "root_canonical_bytes": 768582,
    "root_canonical_sha256": "cb9b512e03d82604270a22cf60e288f9ea8f6aa808517a6319a27f860552bcc0",
    "shared_transaction_id_file": "crates/jocforge-domain/src/transaction/mod.rs",
    "status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__PACKET_PRODUCERS_DEFERRED",
    "test_file": "crates/jocforge-domain/tests/spatial_control_foundation.rs",
    "verified_scope": "complete v5 persistent cold root, full canonical bytes/digest, read-only machine, provider/profile lineage guards, opaque packet capability type reference, CurrentFeatures packet ownership in the separate producer slice, noncold prepare deferred"
  },
  "implementation_status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__PACKET_PRODUCERS_DEFERRED",
  "input_only_vectors": {
    "bytes": 2387,
    "case_count": 14,
    "deferred_cases": [
      {
        "id": "flags_before_history",
        "status": "DEFERRED_TO_SOURCE_FLAGS_PRODUCER"
      },
      {
        "id": "feature_transpose",
        "status": "DEFERRED_TO_CURRENT_FEATURES_PRODUCER"
      },
      {
        "id": "negative_zero_cold_rejected",
        "status": "DEFERRED_TO_TYPED_AFTER_PROPOSAL_VALIDATOR"
      },
      {
        "id": "zero_selection_retains_installed",
        "status": "DEFERRED_TO_F03_TRANSITION_PRODUCER"
      },
      {
        "id": "quiet_changes_committed_active",
        "status": "DEFERRED_TO_F03_TRANSITION_PRODUCER"
      },
      {
        "id": "parent_failure_discards_packets",
        "status": "DEFERRED_TO_PARENT_TRANSACTION_COORDINATOR"
      },
      {
        "id": "reset_same_profile",
        "status": "DEFERRED_TO_PROGRAMME_RESET_COORDINATOR"
      }
    ],
    "expected_values_as_control_flow": "FORBIDDEN",
    "fixture_file": "crates/jocforge-domain/tests/fixtures/spatial_control_state_v5/INPUT-ONLY-VECTORS.json",
    "product_execution": "NOT_RUN",
    "sha256": "c5292f9a4dfd607704f4a442963b56b3f239e0bc69b8a9d03619fc65bdbc0d69"
  },
  "join_metadata": [
    "shared CallbackTransactionId",
    "same FrontendCallbackIndex",
    "selected EncodingProfile identity",
    "before_generation and before_sha256",
    "complete after_sha256 over the full v5 persistent root",
    "validated F01E provider receipt lineage",
    "validated F03 target provider receipt lineage",
    "SourceFlagsPacket and CurrentFeaturesPacket bound to the same parent transaction"
  ],
  "profile_only_mismatch_vector": {
    "bytes": 2318,
    "case_id": "same_hash_rejects_unselected_profile_identity",
    "expected_result": "REJECT_PROFILE_BINDING",
    "fixture_file": "crates/jocforge-domain/tests/fixtures/spatial_control_state_v5/PROFILE-IDENTITY-MISMATCH-VECTORS.json",
    "profile_is_supported": false,
    "same_f01e_receipt_lineage": true,
    "same_f03_receipt_lineage": true,
    "same_generation": true,
    "same_state_hash": true,
    "sha256": "42690e9dc752f9366c95ff321d778c433a2c2686b574a7f8a54e74f2b1e67aa2"
  },
  "programme_root": "spatial_control",
  "prohibitions": [
    "no read-time filesystem or JSON access",
    "no hard-coded provider JSON values",
    "no callback-local scratch or external owner state in the persistent root",
    "no zero-fill for unspecified persistent values",
    "no use of the 630-byte target projection as the full root digest",
    "no persistence or canonical encoding for profile_dimensions",
    "do not implement SourceFlagsPacket or CurrentFeaturesPacket producers",
    "do not expose public raw-array packet constructors or zero/default packets",
    "do not forge packet producer lineage",
    "do not expose a test-only typed-after proposal constructor"
  ],
  "required_verification": [
    "complete v5 cold field values, shapes and canonical digest",
    "provider mismatch and F03 target sentinel/attribute/current-previous independence",
    "same state hash with distinct profile identities and equal F01E/F03 lineages is rejected",
    "same state hash with equal profile identity and a distinct provider lineage is rejected",
    "same-hash profile-only rejection leaves state hash and generation unchanged",
    "full cold root excludes callback packets and profile dimensions; profile dimensions contribute zero bytes and no tag",
    "persistent source_vector and source_scalar have exact F32 shapes and positive-zero bits",
    "flags_before_history is outside D1 and is resolved only by the separate SOURCE_FLAGS_PRODUCER slice; do not attribute that evidence to D1",
    "packet producer capabilities are opaque, non-defaultable, non-forgeable, and NOT_READY at cold/reset",
    "noncold prepare remains crate-private/deferred until a complete producer-generated typed after proposal exists",
    "no test-only constructor exposes the deferred after-proposal path",
    "machine and tokens cannot be cloned or copied",
    "compile-fail proof for public commit/abort/fatal and mutable committed access"
  ],
  "root_schema": "selected-adm-installed48-state.v5",
  "schema_amendment_file": "docs/implementation/SPATIAL-CONTROL-STATE-V5-AMENDMENT.md",
  "scope": "construct the complete v5 persistent spatial_control cold root and canonical digest; provide a non-Clone read-only machine and prepared/staged foundation accepting only a future producer-generated complete typed after proposal; no packet producers and no local commit, abort, fatal or top-level COMMITTED",
  "scope_approval": "user-approved in the current task handoff on 2026-10-16; scope-only, not a packet-producer specification",
  "spec_gap_document": {
    "document_bytes": 2796,
    "document_sha256": "25b37ca27365d4784d35760dadb9ad32476505bdd1919199fd0131ae5e465d2f",
    "file": "docs/implementation/SPATIAL-CONTROL-STAGING-SPEC-GAP.md"
  },
  "status": "LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__PACKET_PRODUCERS_DEFERRED",
  "task_id": "P1.3-D1-SPATIAL-CONTROL-STAGING-ONLY"
}
```

## SourceFlags producer

```json
{
  "authority": "identity-verified approved implementer supplement; error names are isolated-product API choices, not native error-code claims",
  "implementation": {
    "file_inventory": [
      {
        "bytes": 33161,
        "path": "crates/jocforge-domain/src/spatial/source_flags.rs",
        "sha256": "8b3318a4ebdeae5a17a4624aaf5e587b42d6737ecdf79d21b8f53f4a95497d63"
      },
      {
        "bytes": 29657,
        "path": "crates/jocforge-domain/src/spatial/source_flags/source_flags_tests.rs",
        "sha256": "e813e0cf50e7106ed43c3f6261abb1c89f23494ee5ab2b1c5d2a306bbea16e0e"
      },
      {
        "bytes": 27933,
        "path": "crates/jocforge-domain/tests/fixtures/source_flags/INPUT-ONLY-VECTORS.json",
        "sha256": "ba7fb477931bc6f7127b63631b8048e127ecd78514133749ce3b0acdf61d3aaf"
      }
    ],
    "fixture_file": "crates/jocforge-domain/tests/fixtures/source_flags/INPUT-ONLY-VECTORS.json",
    "module_file": "crates/jocforge-domain/src/spatial/source_flags.rs",
    "test_file": "crates/jocforge-domain/src/spatial/source_flags/source_flags_tests.rs",
    "test_mode": "20 synthetic literal vectors in workspace suite; authorized WAVE boundary vector is explicitly invoked with JOCFORGE_AUTHORIZED_WAV and is ignored otherwise"
  },
  "implementation_status": "LOCALLY_VERIFIED_NUMERIC_AND_PACKET",
  "lineage_binding": [
    "programme ID",
    "attempt",
    "CallbackTransactionId",
    "FrontendCallbackIndex",
    "before-history owner generation and digest",
    "selected EncodingProfile identity",
    "validated F01E and F03 provider receipt lineage",
    "current PCM receipt identity",
    "regular control receipt identity",
    "semantic route/LFE receipt identity",
    "prefix writer receipt identity"
  ],
  "numeric_contract": {
    "activity_comparison": "strict >",
    "activity_epsilon_f32_bits": "0x34000000",
    "operation_rounding": "each conversion and multiplication has its specified independent R32 point; no FMA, add, energy or RMS path",
    "output_layout": "source-major U8[48][2], 96 bytes",
    "triangle_window_bytes": 12288,
    "triangle_window_sha256": "134a5a788d42b0532f3ec46112ad978cc14fb3c46cb0d82f02fee5f129b7c919",
    "width_comparison": ">= only when active and unmuted",
    "width_limit_f32_bits": "0x3e4ccccd"
  },
  "output_ownership": {
    "CurrentFeatures_or_FFT_implemented": true,
    "SourceFlagsPacket": "opaque linear non-Clone/non-Copy capability; crate-private producer constructor and read-only payload/lineage access; no Default or cold packet",
    "next_stage_order": "render/writer receipts -> flags -> CurrentFeatures -> selector/outer",
    "parent_abort": "discard packet and proposal without committed history changes",
    "parent_transaction_implemented": false,
    "raw_history_proposal": "exact raw current PCM regular rows in route order; proposal only for the existing owner; no commit or additional state owner",
    "selector_or_outer_transition_implemented": false
  },
  "scope": "pure same-callback SourceFlagsPacket production plus exact raw-current-history proposal for the existing owner; no history commit or parent transaction execution",
  "status": "LOCALLY_VERIFIED_NUMERIC_AND_PACKET",
  "supplement": {
    "files": [
      {
        "bytes": 1060,
        "file": "API-SCHEMA.json",
        "sha256": "4e9d9e343b836943af9975b5b25e41c906faea684a0f794985eb419253665a8a"
      },
      {
        "bytes": 27933,
        "file": "INPUT-ONLY-VECTORS.json",
        "sha256": "ba7fb477931bc6f7127b63631b8048e127ecd78514133749ce3b0acdf61d3aaf"
      },
      {
        "bytes": 4169,
        "file": "SPEC.md",
        "sha256": "6dfccb7edb73391ae61a686c0a84373ab64e4a6a7018a6b9b93ac879e1957500"
      }
    ],
    "identity": "source-flags-producer.v1",
    "manifest_bytes": 1247,
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "164b7d4857922ea7149280968a56fa9206f2fd936e6fd413597548b4b8c70d68",
    "research_basis_files_read": false
  },
  "task_id": "SOURCE-FLAGS-PRODUCER-V1",
  "typed_inputs": [
    "finite host-major current PCM F32[38][1536]",
    "finite before regular history F32[37][1536] in validated regular-host order",
    "exactly 37 typed regular-host controls",
    "validated 38-host semantic route with exactly one excluded LFE and ordered 11..47 destinations",
    "same-callback prefix writer set limited to sources 0..10",
    "typed programme ID, attempt, CallbackTransactionId, FrontendCallbackIndex, before-history generation/digest, selected profile and validated provider lineage"
  ],
  "vectors": {
    "authorized_boundary_execution": "first 1536-sample callback read from externally supplied JOCFORGE_AUTHORIZED_WAV; exact 96-byte vector matched",
    "authorized_boundary_success_count": 1,
    "authorized_wav_path_or_pcm_stored": false,
    "authorized_wav_sha256": "7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1",
    "bytes": 27933,
    "error_case_ids": [
      "stale_packet",
      "next_history_wrong_generation",
      "disabled_route",
      "wrong_PCM_shape",
      "nonbijective_mapping",
      "missing_before_history",
      "unsupported_component",
      "unsupported_diffuse",
      "NaN_current_input",
      "NaN_width",
      "invalid_prefix_writer"
    ],
    "error_count": 11,
    "error_output": "null packet and zero committed history changes",
    "fixture_file": "crates/jocforge-domain/tests/fixtures/source_flags/INPUT-ONLY-VECTORS.json",
    "sha256": "ba7fb477931bc6f7127b63631b8048e127ecd78514133749ce3b0acdf61d3aaf",
    "success_case_ids": [
      "cold_silent_no_writer",
      "current_pulse_host0",
      "negative_current_pulse",
      "history_pulse_before",
      "epsilon_product_equal",
      "epsilon_product_below",
      "epsilon_product_above",
      "zero_gain_last_current",
      "zero_gain_first_history",
      "negative_zero_silent",
      "mute_active_keeps_raw_history",
      "mute_silent",
      "width_below_point2_active",
      "width_equal_point2_active",
      "width_above_point2_active",
      "width_equal_point2_inactive",
      "prefix_writer_on_silence",
      "LFE_pulse_excluded",
      "semantic_LFE_rebound_host10",
      "prefix_duplicate_set_member",
      "real_first_callback_case"
    ],
    "success_count": 21,
    "synthetic_success_count": 20,
    "whole_song_product_replay": "NOT_RUN"
  }
}
```

## P1.4-A F01E provider contract audit

```json
{
  "boundaries": {
    "application": "coordinate I/O evidence to domain validation; never bypass receipt",
    "domain": "recompute MANIFEST SHA before parse, derive the unique provider entry, validate provider bytes against it, then parse/validate/canonicalize the provider; no file access",
    "io": "read immutable MANIFEST/provider bytes and report diagnostic path/observed length/SHA only; no JSON interpretation"
  },
  "canonical_reconstruction": {
    "cold_preimage_hex_source": "parameters/f01e-selected-constructor-v1.json#/cold_preimage_hex",
    "ensure_ascii": false,
    "expected_bytes": 4402,
    "expected_sha256": "9c943879cd36cb45bc7538346cfdf4f71d0998d8f261f75c135045fcb59870aa",
    "number_rule": "decimal integers only; lowercase true/false/null",
    "relations": [
      "manifest SHA authenticates complete provider file bytes",
      "cold_preimage_hex is expected canonical /value bytes",
      "cold_sha256 authenticates reconstructed canonical /value bytes"
    ],
    "separators": [
      ",",
      ":"
    ],
    "sort_keys": true,
    "trailing_bytes": "none",
    "writer": "custom typed F01E canonical writer; do not rely on generic map serializer"
  },
  "dependency_candidate": {
    "archive_bytes": 158079,
    "archive_sha256": "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6",
    "cached_archive": "C:/Users/chyinan/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/serde_json-1.0.154.crate",
    "crate": "serde_json",
    "custom_required": [
      "typed exact shape/type/range validator",
      "custom duplicate-key-rejecting visitor as defense in depth",
      "custom typed canonical writer"
    ],
    "exact_version_candidate": "=1.0.154",
    "features_candidate": {
      "default_features": false,
      "features": [
        "std"
      ]
    },
    "license": "MIT OR Apache-2.0",
    "not_relied_on": [
      "duplicate-key policy before identity",
      "unknown-field policy before identity",
      "canonical key ordering",
      "canonical writer",
      "arbitrary precision numbers"
    ],
    "repository": "https://github.com/serde-rs/json",
    "required_capabilities": [
      "UTF-8 JSON syntax parse after identity gate",
      "integer/string/bool/null access",
      "custom visitor duplicate-field rejection"
    ],
    "status": "APPROVED_AND_LOCKED"
  },
  "errors": [
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PACKAGE_MANIFEST_OBSERVATION_MISMATCH"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PACKAGE_MANIFEST_IDENTITY_MISMATCH"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PACKAGE_MANIFEST_ENTRY_MISMATCH"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PROVIDER_LENGTH_MISMATCH"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PROVIDER_SHA256_MISMATCH"
    },
    {
      "category": "INPUT_REJECTION",
      "code": "PROVIDER_UTF8_ERROR"
    },
    {
      "category": "INPUT_REJECTION",
      "code": "PROVIDER_JSON_SYNTAX"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PROVIDER_SCHEMA_MISMATCH"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PROVIDER_SHAPE_TYPE_RANGE"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PROVIDER_CANONICAL_PREIMAGE_MISMATCH"
    },
    {
      "category": "PARAMETER_IDENTITY",
      "code": "PROVIDER_CANONICAL_DIGEST_MISMATCH"
    }
  ],
  "failure": "no provider receipt/state; caller fail-stop before P1.3-B stage",
  "implementation_status": "LOCALLY_VERIFIED",
  "input_acceptance": {
    "bom": "rejected by identity mismatch",
    "duplicate_keys": "any modified duplicate-key document rejected by identity mismatch before parser; typed visitor must also reject duplicates defensively",
    "exact_bytes_only": true,
    "field_order": "only exact provider byte order accepted; canonical output independently sorts keys",
    "number_syntax": "alternative numeric spellings rejected by identity mismatch; typed values require exact integer ranges and no floats",
    "ordering": "identity gate before decode/parse",
    "trailing_whitespace_or_bytes": "rejected by length/SHA identity mismatch",
    "unicode": "exact UTF-8 bytes accepted; escape/spelling changes rejected by identity mismatch",
    "unknown_fields": "any modified unknown-field document rejected by identity mismatch; typed validation rejects unknowns defensively",
    "utf8": "exact asset must decode as UTF-8; decode failure after identity is PROVIDER_UTF8_ERROR"
  },
  "io_input": {
    "shared_fields": [
      "immutable Arc<[u8]> bytes",
      "optional diagnostic path; never identity",
      "observed byte length",
      "observed SHA-256"
    ],
    "types": [
      "PackageManifestEvidence",
      "ProviderFileEvidence"
    ],
    "validation_flow": [
      "recompute complete MANIFEST.json SHA-256 before parsing",
      "parse the SHA-authenticated manifest and require one unique F01E entry",
      "bind entry path, bytes and SHA-256 into ValidatedF01EPackageManifest",
      "validate provider observation and bytes against that validated entry"
    ]
  },
  "output": {
    "receipt": {
      "fields": [
        "validated_manifest_lineage",
        "provider_file_sha256",
        "provider_file_bytes",
        "schema_id",
        "schema_version",
        "value_pointer",
        "canonical_sha256",
        "canonical_bytes",
        "provider_generation"
      ],
      "immutability": "all fields immutable",
      "lineage": "validated_manifest_lineage contains expected ProviderIdentity plus the recomputed SHA-256 from ValidatedF01EPackageManifest; caller constants are not proof",
      "provider_generation": 0,
      "type": "F01EProviderReceipt"
    },
    "validated_provider": "ValidatedF01EColdProvider with private typed fields; no serde_json::Value exposure"
  },
  "provider_identity": {
    "equality": "all fields exact; diagnostic path excluded",
    "fields": {
      "delivery_scope": "INTERNAL_RESEARCH_AND_INTERNAL_IMPLEMENTATION_ONLY",
      "manifest_entry_bytes": 20138,
      "manifest_entry_path": "parameters/f01e-selected-constructor-v1.json",
      "manifest_entry_sha256": "b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb",
      "package_manifest_path": "MANIFEST.json",
      "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
      "provider_id": "F01E_SELECTED_COLD_V1",
      "schema_id": "f01e-selected-constructor.v1",
      "schema_version": 1
    },
    "rejection": "any identity field mismatch rejects before UTF-8/JSON parse",
    "type": "ProviderIdentity<F01E_SELECTED_COLD_V1>"
  },
  "scope": "F01E_SELECTED_COLD_V1 provider identity, typed parse, validation and receipt only",
  "shape_validation": {
    "bindings": "dbmd/physical null at cold",
    "normalized_config_fields": [
      "sample_rate_hz",
      "channels",
      "pcm_bits",
      "source_samples",
      "bit_rate_kbps",
      "bit_rate_enum",
      "layout_enum",
      "target_capacity",
      "frame_samples",
      "spatial_config_mode",
      "dispatch_mode",
      "virtual_prefix_count",
      "bed_count",
      "object_count",
      "event_start_samples",
      "line_drc",
      "rf_drc",
      "time_base"
    ],
    "objects": "exact array[28], ordinal 0..27 unique/in order, host/event null cold, sampler false, xyz_u32[3] zero",
    "pointer": "/value",
    "ranges": "all selected cold constants exact; bool is not integer",
    "snapshots": "control38 null or U32[38][13]; regular37 null or U32[37][13]",
    "top_level_fields": [
      "schema",
      "callback_index",
      "normal_callbacks",
      "remaining_cached_drains",
      "phase",
      "objects",
      "last_control38_snapshot",
      "last_regular37_snapshot",
      "dbmd_binding",
      "physical_binding",
      "normalized_config"
    ]
  },
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.4-A-F01E-PROVIDER-CONTRACT"
}
```

## P1.4-B F03 selected target cold provider mapping

```json
{
  "boundaries": {
    "application": "coordinate evidence and validated provider receipt only; no spatial_control staging or commit",
    "domain": "validate provider identity, parse exact fields, map the eight cold target arrays and form the target-only projection; no file access",
    "io": "read immutable package manifest and provider bytes; diagnostic paths are not identity"
  },
  "dependency_scope": {
    "allowed_use": "F03 provider syntax parsing and private field access after identity gates only",
    "archive_sha256": "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6",
    "canonical_writer": "project-owned typed writer; serde_json serialization is not used",
    "crate": "serde_json",
    "default_features": false,
    "features": [
      "std"
    ],
    "version": "=1.0.154"
  },
  "failure": "identity, syntax, shape, field, cold-value or projection mismatch yields no validated provider or receipt; caller fails before P1.3-D staging",
  "field_invariants": [
    "current and previous target arrays are distinct logical state fields; equal cold values do not permit mutable aliasing",
    "0xFFFFFFFF is the valid cold target-ID sentinel; do not apply warm-ID range 0..47 validation to cold IDs",
    "0x00000B00 is the selected target cold attribute; do not substitute reader or metadata cold attribute 0",
    "active_u8 remains U8[15]; do not widen it to a U32 ABI field"
  ],
  "implementation": {
    "fixture_bytes": 3139,
    "fixture_file": "crates/jocforge-domain/tests/fixtures/target_cold/f03-selected-target-cold-v1.json",
    "fixture_sha256": "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00",
    "module_file": "crates/jocforge-domain/src/frontend/target_cold_provider.rs",
    "public_exports_file": "crates/jocforge-domain/src/frontend/mod.rs",
    "spatial_control_implementation": "NOT_IMPLEMENTED",
    "test_file": "crates/jocforge-domain/tests/target_cold_provider.rs",
    "verified_scope": "manifest/provider identity gates, exact target arrays, independent current/previous storage, target-only projection and immutable lineage receipt"
  },
  "implementation_prohibitions": [
    "no provider-backed spatial_control staging or commit in P1.4-B",
    "no spatial_control production implementation",
    "target_projection_sha256 is not a full spatial_control root digest",
    "no private key or external owner state"
  ],
  "implementation_status": "LOCALLY_VERIFIED",
  "io_input": {
    "diagnostic_path_is_identity": false,
    "identity_first": true,
    "types": [
      "PackageManifestEvidence",
      "ProviderFileEvidence"
    ],
    "validation_flow": [
      "recompute and match the complete MANIFEST.json SHA-256 before parsing the manifest",
      "then validate observed manifest length and SHA-256 against the actual bytes",
      "parse the authenticated manifest and require one unique F03 provider entry with exact path, byte length and SHA-256",
      "recompute provider bytes and validate observed length and SHA-256 plus the unique manifest entry before UTF-8 or JSON parsing",
      "any changed provider bytes, including duplicate-key variants, fail identity before parse; then reject malformed UTF-8, JSON syntax, duplicate keys and unknown top-level fields",
      "validate exact field types, fixed array lengths and cold-value constraints before producing a validated provider"
    ]
  },
  "output": {
    "receipt": {
      "authority": "internal immutable receipt shape following the approved P1.4-A provider-lineage pattern; not a package field schema",
      "canonical_scope": "the 630-byte target-only projection above; not the complete spatial_control root",
      "fields": [
        "validated_manifest_lineage",
        "provider_file_sha256",
        "provider_file_bytes",
        "schema_id",
        "target_projection_sha256",
        "target_projection_bytes",
        "provider_generation"
      ],
      "immutability": "all fields immutable",
      "lineage": "validated_manifest_lineage binds the expected provider identity and recomputed manifest entry; caller constants alone are not proof",
      "provider_generation": 0,
      "type": "F03TargetProviderReceipt"
    },
    "validated_provider": "ValidatedF03TargetColdProvider with eight private typed arrays, read-only accessors, and no serde_json::Value exposure"
  },
  "prerequisites": [
    {
      "status": "LOCALLY_VERIFIED",
      "task_id": "P1.4-A-F01E-PROVIDER-CONTRACT"
    }
  ],
  "profile_dimensions": {
    "active_target_count_n": {
      "cold_numeric_value": null,
      "state_field": false,
      "type": "explanatory string",
      "value": "runtime integer 0..15; arrays allocate T=15 and consume prefix [0:n]"
    },
    "axes": 3,
    "feature_bands_B": 24,
    "source_count_N": 48,
    "target_capacity_T": 15
  },
  "provider_identity": {
    "additional_or_unknown_top_level_fields": "REJECT",
    "manifest_entry_bytes": 3139,
    "manifest_entry_sha256": "a4b53854822a5f17a5413d6f2ded127a7366257c2a171377afcea4a84a3eaf00",
    "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "path": "parameters/f03-selected-target-cold-v1.json",
    "private_key_required": false,
    "provider_kind": "exact_parseable_functional_constant",
    "schema_id": "f03-selected-target-cold.v1"
  },
  "scope": "F03 selected target cold provider identity, exact shape, committed target-field mapping, and target-only canonical projection; provider implementation and tests only",
  "status": "LOCALLY_VERIFIED",
  "target_only_projection": {
    "bytes": 630,
    "encoding": "concatenate fixed arrays in row-major order; U32 fields little-endian; XYZ values are raw F32 U32 bits; U8 fields are unmodified bytes; no header, tags, counts, padding, or trailing bytes",
    "excluded": [
      "profile_dimensions.active_target_count_n explanatory string",
      "callback-local selector, refinement, NNLS and outer scratch",
      "provider source, evidence and local_review metadata",
      "external F05R56, F04C, F05JL, F13 and F15 owner state"
    ],
    "field_order": [
      "current_xyz_u32",
      "current_id_u32",
      "current_attribute_u32",
      "previous_xyz_u32",
      "previous_id_u32",
      "previous_attribute_u32",
      "active_u8",
      "previous_marker_active_u8"
    ],
    "offsets": [
      {
        "bytes": 180,
        "field": "current_xyz_u32",
        "offset": 0
      },
      {
        "bytes": 60,
        "field": "current_id_u32",
        "offset": 180
      },
      {
        "bytes": 60,
        "field": "current_attribute_u32",
        "offset": 240
      },
      {
        "bytes": 180,
        "field": "previous_xyz_u32",
        "offset": 300
      },
      {
        "bytes": 60,
        "field": "previous_id_u32",
        "offset": 480
      },
      {
        "bytes": 60,
        "field": "previous_attribute_u32",
        "offset": 540
      },
      {
        "bytes": 15,
        "field": "active_u8",
        "offset": 600
      },
      {
        "bytes": 15,
        "field": "previous_marker_active_u8",
        "offset": 615
      }
    ],
    "scope": "target-only cold-state identity; not the complete spatial_control root state or root digest",
    "sha256": "b1acfa3ce43b6ce987836330a3db3945c81b3d62d96d4973f18b302dc6d2606c"
  },
  "target_state_fields": [
    {
      "canonical_bytes": 180,
      "canonical_encoding": "U32LE",
      "canonical_offset": 0,
      "cold": "45 x 0x00000000",
      "index_mapping": "slot*3+X/Y/Z",
      "maps_to": "selection_outer.target_current_xyz F32[15][3] raw U32 bits",
      "provider_field": "current_xyz_u32",
      "type": "flat U32[45]"
    },
    {
      "canonical_bytes": 60,
      "canonical_encoding": "U32LE",
      "canonical_offset": 180,
      "cold": "15 x 0xFFFFFFFF",
      "maps_to": "selection_outer.target_current_id",
      "provider_field": "current_id_u32",
      "type": "U32[15]"
    },
    {
      "canonical_bytes": 60,
      "canonical_encoding": "U32LE",
      "canonical_offset": 240,
      "cold": "15 x 0x00000B00 (2816)",
      "maps_to": "selection_outer.target_current_attribute",
      "provider_field": "current_attribute_u32",
      "type": "U32[15]"
    },
    {
      "canonical_bytes": 180,
      "canonical_encoding": "U32LE",
      "canonical_offset": 300,
      "cold": "45 x positive-zero bits 0x00000000",
      "index_mapping": "slot*3+X/Y/Z",
      "maps_to": "selection_outer.target_previous_xyz F32[15][3] raw U32 bits",
      "provider_field": "previous_xyz_u32",
      "type": "flat U32[45]"
    },
    {
      "canonical_bytes": 60,
      "canonical_encoding": "U32LE",
      "canonical_offset": 480,
      "cold": "15 x 0xFFFFFFFF",
      "maps_to": "selection_outer.target_previous_id",
      "provider_field": "previous_id_u32",
      "type": "U32[15]"
    },
    {
      "canonical_bytes": 60,
      "canonical_encoding": "U32LE",
      "canonical_offset": 540,
      "cold": "15 x 0x00000B00 (2816)",
      "maps_to": "selection_outer.target_previous_attribute",
      "provider_field": "previous_attribute_u32",
      "type": "U32[15]"
    },
    {
      "canonical_bytes": 15,
      "canonical_encoding": "U8",
      "canonical_offset": 600,
      "cold": "15 x 0",
      "maps_to": "selection_outer.target_active",
      "provider_field": "active_u8",
      "type": "U8[15]"
    },
    {
      "canonical_bytes": 15,
      "canonical_encoding": "U8",
      "canonical_offset": 615,
      "cold": "15 x 0",
      "maps_to": "selection_outer.previous_marker_active",
      "provider_field": "previous_marker_active_u8",
      "type": "U8[15]"
    }
  ],
  "task_id": "P1.4-B-F03-SELECTED-TARGET-COLD-PROVIDER-CONTRACT",
  "top_level_fields": [
    "schema",
    "profile_dimensions",
    "provider_kind",
    "current_xyz_u32",
    "current_id_u32",
    "current_attribute_u32",
    "previous_xyz_u32",
    "previous_id_u32",
    "previous_attribute_u32",
    "active_u8",
    "previous_marker_active_u8",
    "source",
    "private_key_required",
    "evidence",
    "local_review"
  ],
  "wash_side_evidence": {
    "authority": "approved clean-room mapping basis; not a product-execution claim",
    "field_mapping_sha256": "8e7e8c8f96abc221f110fbf4fc618c6850682939b83cf11796e145344d399c5b",
    "findings_sha256": "851782d973fc40099099a4e76e9ef53399664c2eafa9ce023b120c407b474699",
    "input_only_vectors_sha256": "c7b451b2d49a358605e31c271d41a21d3a50b204f5043259b04083521a3288b9",
    "loader_staging_full_root_and_audio_execution": "NOT_RUN",
    "private_key_present": false,
    "research_validation": "12 semantic mutations; 14 source pins; PASS_RESEARCH_MATERIAL_STRUCTURE_AND_IDENTITIES_ONLY",
    "schema_basis_sha256": "9ac88011b7e95546b977b2119d4f8b52b8a2d976e39d6edecb812e048d339790"
  }
}
```

## P1.4-C CurrentFeatures math provider mapping

```json
{
  "current_features_producer": {
    "algorithm_rust": "LOCALLY_VERIFIED_IN_CURRENT_FEATURES_PRODUCER_SLICE",
    "branches": {
      "00": "READ_AUDIO_INPUT",
      "01": "RESET; consumer priority branch tested, not emitted by SourceFlags producer",
      "01_product_boundary_rule": "accepted as reset-priority consumer input; no SourceFlags producer output claim",
      "10": "CARRY",
      "audio_read": "only READ_AUDIO_INPUT reads sample payloads or calls FFT; RESET/CARRY do not read audio",
      "flag_pair_order": [
        "processing_inactive",
        "selection_suppressed"
      ]
    },
    "consumer_boundary_open_questions": [],
    "implementation": {
      "fanout_mapping_document": "docs/implementation/POSTWINDOW-FANOUT-MAP.md",
      "literal_vector_fixture": "crates/jocforge-domain/tests/fixtures/current_features/INPUT-ONLY-LITERAL-VECTORS.json",
      "literal_vector_fixture_bytes": 315386,
      "literal_vector_fixture_sha256": "1aaf1f5296569960e188b8d74fc8097b5bfb6099b5a1079366e595011c87bdc3",
      "mapping_document": "docs/implementation/CURRENT-FEATURES-PRODUCER-MAP.md",
      "module_file": "crates/jocforge-domain/src/spatial/current_features.rs",
      "parent_composer_or_commit_implemented": false,
      "post_window_fanout_contract_file": "crates/jocforge-domain/src/spatial/post_window_fanout.rs",
      "post_window_fanout_test_file": "crates/jocforge-domain/src/spatial/post_window_fanout/tests.rs",
      "render_producer_implemented": false,
      "unit_test_file": "crates/jocforge-domain/src/spatial/current_features/tests.rs"
    },
    "implementation_status": "LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED",
    "integration_contract": {
      "active_post_window_fanout": {
        "expected_callback_phase": "explicit CallbackPhase; never inferred from payload bits",
        "proposal": "PostWindowRenderProposal",
        "render_producer_implemented": false,
        "views": [
          "CurrentFeaturesAudioView",
          "SpatialAudioPostWindowView"
        ]
      },
      "api_schema_bytes": 786,
      "api_schema_file": "API-SCHEMA.json",
      "api_schema_sha256": "280ebef7034797e7efa84e2e3514cbde05ffd973ac644c5f13e804d057b778c6",
      "authority": "wash-side approved isolated product API; not native receipt compatibility",
      "before_feature_identity": [
        "SpatialControlMachine.generation",
        "SHA-256 of complete committed v5 root canonical bytes"
      ],
      "contract_bytes": 3919,
      "contract_file": "CONTRACT.md",
      "contract_sha256": "3cf97a638fbe02dc36edf8a29534493e2b7de8947b858f749eefdef636cae0a5",
      "deferred": [
        "post-window render producer",
        "matrix/destination producer",
        "future spatial-audio input assembler",
        "complete spatial-control after-state composer",
        "parent transaction executor",
        "selector/outer",
        "real ADM replay",
        "codec/EC3",
        "perceptual acceptance"
      ],
      "error_outcomes": {
        "error_names_are_native_codes": false,
        "identity_lineage_shape_profile_provider_or_preflight_mismatch": "reject before numeric work; no capability; committed state unchanged",
        "numeric_nonfinite_or_overflow": "no output or partial proposal; parent applies existing GTC fatal/retirement policy",
        "parent_failure_after_production": "discard linear capability; committed state unchanged"
      },
      "identity": "current-features-integration-contract.v1",
      "manifest_bytes": 727,
      "manifest_file": "MANIFEST.json",
      "manifest_sha256": "e240701f0128877baefb305152f10553de9288dc05298452d3605011e63603ad",
      "output_capability": {
        "contains": [
          "FeatureStateProposal source vector F32[48][24]",
          "FeatureStateProposal source scalar F32[48]",
          "opaque CurrentFeaturesPacket F32[24][48] bitwise transpose",
          "immutable copied callback phase/render-stage/after-proposal/payload lineage and exact math-provider receipts",
          "existing SourceFlagsPacket for future spatial composer"
        ],
        "future_consumer": "spatial-control transition composer only",
        "local_commit_abort_fatal_or_persistent_packet": false,
        "type": "linear CurrentFeaturesProduced"
      },
      "source_flags_validation": "existing opaque SourceFlagsPacket; crate-private read-only payload/lineage validation against parent, CurrentFeaturesAudioView, selected profile, provider lineage, and spatial-control before identity; no new public receipt or digest",
      "superseded_by": {
        "identity": "postwindow-fanout-contract.v1",
        "manifest_sha256": "2a55e66314d6f2aa37cf792b62068185358b1a4b294e6c7b95e8b8d77d99f107"
      },
      "superseded_post_window_packet_contract": {
        "binding": [
          "programme",
          "attempt",
          "CallbackTransactionId",
          "FrontendCallbackIndex",
          "selected EncodingProfile",
          "validated F01E/F03 provider lineage",
          "spatial-control before generation and complete-root digest",
          "render proposal/receipt identity",
          "SHA-256 of exactly payload F32 U32LE bytes"
        ],
        "boundary": "INSTALLED48_POST_WINDOW_48X3072",
        "construction_authority": "future installed48/render producer only",
        "current_features_test_fixture_constructor": "module-private cfg(test) only",
        "payload": "finite F32[48][3072] source-major",
        "public_raw_constructor": false,
        "type": "opaque linear non-Clone PostWindowAudioPacket"
      }
    },
    "lineage_binding": [
      "programme",
      "attempt",
      "CallbackTransactionId",
      "FrontendCallbackIndex",
      "CallbackPhase",
      "before feature generation/digest",
      "spatial-control after-proposal identity when a view is supplied",
      "render-stage identity when a view is supplied",
      "payload byte count and SHA-256 when a view is supplied",
      "flags receipt",
      "selected profile identity",
      "validated F01E/F03 provider lineage",
      "validated feature-processor math provider receipt",
      "validated FFT roots provider receipt"
    ],
    "outputs": {
      "CurrentFeaturesPacket": "opaque F32[24][48] band-major; bitwise transpose of after vector",
      "FeatureStateProposal": "F32[48][24] source-major plus F32[48] scalar",
      "lineage": "same validated parent identity and explicit phase; render-stage/after-proposal identities, payload SHA-256 and byte count are optional and present only when a view is supplied; exact math-provider receipts are always present",
      "local_commit": false,
      "parent_commit_required": true,
      "persistent_packet": false,
      "producer": "linear CurrentFeaturesProduced",
      "source_flags_packet_preserved": true
    },
    "parent_transaction_or_product_execution": "NOT_RUN",
    "raw_provider_evidence_consumption": "FORBIDDEN",
    "required_context_shapes": {
      "before_source_scalar": "F32[48]",
      "before_source_vector": "F32[48][24] source-major",
      "callback_phase": "explicit CallbackPhase NORMAL or CACHED_DRAIN; never inferred from audio bits",
      "post_window_audio": "Option<CurrentFeaturesAudioView>; required when any source is READ (00), None is allowed when all sources are RESET/CARRY",
      "source_flags": "same-callback SourceFlagsPacket U8[48][2]"
    },
    "required_provider_types": [
      "ValidatedFeatureProcessorMathProvider",
      "ValidatedFeatureFft4096MathProvider"
    ],
    "selected_route": "REAL_FFT4096_RADIX2_MATH_V1",
    "source_flags_pair_domain": [
      "00",
      "10",
      "11"
    ],
    "status": "LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED",
    "task_id": "CURRENT-FEATURES-PRODUCER-V1",
    "vectors": {
      "bytes": 315386,
      "file": "INPUT-ONLY-LITERAL-VECTORS.json",
      "literal_expected_used_as_input": false,
      "negative_count": 9,
      "negative_error_labels": [
        "READ_AUDIO_MISSING",
        "RENDER_PAYLOAD_SHAPE",
        "RENDER_PAYLOAD_SHAPE",
        "FLAG_DOMAIN",
        "BEFORE_VECTOR_SHAPE",
        "NONFINITE_BEFORE",
        "RENDER_NONFINITE_PAYLOAD",
        "STALE_CALLBACK",
        "NONFINITE_INTERMEDIATE"
      ],
      "negative_outcome": {
        "packet": null,
        "state_commit_count": 0
      },
      "positive_count": 15,
      "sha256": "1aaf1f5296569960e188b8d74fc8097b5bfb6099b5a1079366e595011c87bdc3",
      "verified_output_f32_words": 35280
    }
  },
  "provider_contracts": {
    "algorithm_rust": "FORBIDDEN_IN_P1.4-C_PROVIDER_VALIDATOR_SLICE",
    "api_schema_identity": {
      "bytes": 1291,
      "file": "API-SCHEMA.json",
      "sha256": "6841baa2f7435529316b16388c3a106fadebab7cff3b8f8ecdf394963cdd27d6"
    },
    "base_package_identity": {
      "manifest_bytes": 76302,
      "manifest_file": "MANIFEST.json",
      "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
    },
    "consumer_vectors": {
      "bytes": 315386,
      "errors_are_native_codes": false,
      "file": "INPUT-ONLY-LITERAL-VECTORS.json",
      "negative_count": 9,
      "negative_error_labels": [
        "READ_AUDIO_MISSING",
        "READ_AUDIO_SHAPE",
        "SOURCE_SHAPE",
        "FLAG_DOMAIN",
        "BEFORE_VECTOR_SHAPE",
        "NONFINITE_BEFORE",
        "NONFINITE_AUDIO",
        "STALE_CALLBACK",
        "NONFINITE_INTERMEDIATE"
      ],
      "negative_outcome": {
        "packet": null,
        "state_commit_count": 0
      },
      "per_positive_case_output_bytes": {
        "CurrentFeaturesPacket": 4608,
        "after_source_scalar": 192,
        "after_source_vector": 4608
      },
      "positive_case_ids": [
        "cold_suppress",
        "warm_reset_11",
        "consumer_reset_priority_01",
        "carry_recompute_scalar",
        "read_zero_threshold_equality",
        "read_impulse_no_extra_window",
        "read_dyadic_comb",
        "two_sources_packet_order",
        "read_repeat_blend_continuation",
        "read_12x_amplitude_jump",
        "read_zero_with_warm_history",
        "threshold_below",
        "threshold_first_nonjump",
        "threshold_above",
        "reset_then_read_replay"
      ],
      "positive_count": 15,
      "route": "REAL_FFT4096_RADIX2_MATH_V1",
      "sha256": "1aaf1f5296569960e188b8d74fc8097b5bfb6099b5a1079366e595011c87bdc3",
      "verified_output_f32_words": 35280,
      "whole_product_execution": "NOT_RUN"
    },
    "dependency_scope": {
      "allowed_use": "identity-gated manifest/provider JSON parsing and private typed field access only",
      "archive_sha256": "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6",
      "canonical_writer": "none; exact file identity only",
      "crate": "serde_json",
      "default_features": false,
      "features": [
        "std"
      ],
      "public_serde_json_value": false,
      "version": "=1.0.154"
    },
    "direct_unvalidated_provider_consumption": "FORBIDDEN",
    "errors": {
      "authority": "isolated provider-validator API categories; not native error codes",
      "fft_provider": [
        "ObservedManifestEvidenceMismatch",
        "PackageManifestIdentityMismatch",
        "PackageManifestSyntax",
        "PackageManifestEntry",
        "ObservedProviderEvidenceMismatch",
        "ProviderIdentityMismatch",
        "Utf8",
        "JsonSyntax",
        "Schema",
        "Shape",
        "RouteMismatch"
      ],
      "processor_provider": [
        "ObservedManifestEvidenceMismatch",
        "PackageManifestIdentityMismatch",
        "PackageManifestSyntax",
        "PackageManifestEntry",
        "ObservedProviderEvidenceMismatch",
        "ProviderIdentityMismatch",
        "Utf8",
        "JsonSyntax",
        "Schema",
        "Shape"
      ]
    },
    "fft_quality": {
      "bytes": 3028,
      "file": "FFT-NUMERICAL-QUALITY.json",
      "maximum_sampled_relative_l2_error": 1.2336887697203963e-07,
      "perceptual_or_real_ADM_acceptance": "NOT_RUN",
      "reference": "NumPy float64 mathematical rfft first 2048 bins; not native",
      "sha256": "f17440c8ce7534de6baafdee1077c7d7b5d0e8f2fbeaebb30341c018329ccd8d",
      "synthetic_case_count": 5,
      "universal_error_bound": false
    },
    "fixture_files": [
      "crates/jocforge-domain/tests/fixtures/current_features/feature-processor-math-v1.json",
      "crates/jocforge-domain/tests/fixtures/current_features/feature-fft4096-math-v1.json",
      "crates/jocforge-domain/tests/fixtures/current_features/README.md"
    ],
    "implementation_status": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS",
    "io_input": {
      "diagnostic_path_is_identity": false,
      "domain_file_io": false,
      "identity_first": true,
      "types": [
        "PackageManifestEvidence",
        "ProviderFileEvidence"
      ],
      "validation_flow": [
        "recompute and verify complete package MANIFEST.json byte length/SHA before parse",
        "require exactly one manifest entry per exact provider path with exact bytes/SHA",
        "verify actual provider byte length/SHA before UTF-8 or JSON parse",
        "reject duplicate/unknown fields and exact schema/shape/range/route drift",
        "return only the relevant private typed provider and immutable receipt"
      ]
    },
    "manifest_fixture": "crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json",
    "mapping_document": "docs/implementation/CURRENT-FEATURES-PROVIDER-MAP.md",
    "not_run": [
      "CurrentFeatures algorithm Rust",
      "real ADM math route",
      "codec/EC3",
      "perceptual acceptance",
      "parent transaction executor",
      "old SIMD comparison"
    ],
    "old_simd_route": "NOT_SELECTED_NO_FALLBACK",
    "provider_contracts": [
      {
        "bytes": 35441,
        "identity_rule": "full package-manifest SHA plus exactly one manifest entry and exact provider file byte length/SHA; then exact typed shape validation",
        "metadata_fields": {
          "real_ADM_new_route_acceptance": "NOT_RUN",
          "source": "MATHEMATICAL_L3072_FS48000_GENERATION"
        },
        "path": "parameters/feature-processor-math-v1.json",
        "provider_canonical_preimage": "NOT_PRESENT; no provider projection digest or canonical writer is defined",
        "provider_id": "FEATURE_PROCESSOR_MATH_V1",
        "receipt": {
          "fields": [
            "validated_manifest_lineage",
            "provider_file_sha256",
            "provider_file_bytes",
            "schema_id"
          ],
          "type": "FeatureProcessorMathProviderReceipt"
        },
        "route": null,
        "runtime_fields": [
          {
            "field": "window_length",
            "type": "exact JSON integer",
            "value": 3072
          },
          {
            "field": "sample_rate",
            "type": "exact JSON integer",
            "value": 48000
          },
          {
            "field": "fft_length",
            "type": "exact JSON integer",
            "value": 4096
          },
          {
            "field": "component_count",
            "type": "exact JSON integer",
            "value": 2048
          },
          {
            "field": "vector_count",
            "type": "exact JSON integer",
            "value": 24
          },
          {
            "field": "coefficient_u32",
            "shape": [
              2048
            ],
            "type": "U32 raw F32 bit words"
          },
          {
            "field": "window_counts",
            "shape": [
              24
            ],
            "sum": 2048,
            "type": "exact JSON integer counts",
            "values": [
              4,
              5,
              6,
              8,
              9,
              11,
              14,
              17,
              20,
              25,
              30,
              37,
              45,
              54,
              66,
              80,
              97,
              118,
              144,
              175,
              213,
              259,
              315,
              296
            ]
          },
          {
            "field": "fast_decay_u32",
            "type": "U32 raw F32 bits",
            "value": "0x3F06FCA3"
          },
          {
            "field": "slow_retain_u32",
            "type": "U32 raw F32 bits",
            "value": "0x3F4ED1AB"
          }
        ],
        "schema_id": "feature-processor-math-parameters.v1",
        "schema_id_version_annotation": {
          "provider_json_field": false,
          "source": "schema_id suffix",
          "value": "v1"
        },
        "sha256": "c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b",
        "top_level_fields": [
          "schema",
          "source",
          "window_length",
          "sample_rate",
          "fft_length",
          "component_count",
          "vector_count",
          "coefficient_u32",
          "window_counts",
          "fast_decay_u32",
          "slow_retain_u32",
          "real_ADM_new_route_acceptance"
        ],
        "typed_output": "ValidatedFeatureProcessorMathProvider with private typed fields and read-only accessors; no serde_json::Value exposure"
      },
      {
        "bytes": 279305,
        "identity_rule": "full package-manifest SHA plus exactly one manifest entry and exact provider file byte length/SHA; then exact typed shape and selected route validation",
        "metadata_fields": {
          "real_ADM_new_route_acceptance": "NOT_RUN",
          "source": "MATHEMATICAL_ROOT_GENERATION_NO_COMPILED_TABLE_INPUT"
        },
        "path": "parameters/feature-fft4096-math-v1.json",
        "provider_canonical_preimage": "NOT_PRESENT; no provider projection digest or canonical writer is defined",
        "provider_id": "FEATURE_FFT4096_MATH_V1",
        "receipt": {
          "fields": [
            "validated_manifest_lineage",
            "provider_file_sha256",
            "provider_file_bytes",
            "schema_id",
            "route"
          ],
          "type": "FeatureFft4096MathProviderReceipt"
        },
        "route": "REAL_FFT4096_RADIX2_MATH_V1",
        "runtime_fields": [
          {
            "field": "fft_length",
            "type": "exact JSON integer",
            "value": 4096
          },
          {
            "field": "stages",
            "lengths": [
              2,
              4,
              8,
              16,
              32,
              64,
              128,
              256,
              512,
              1024,
              2048,
              4096
            ],
            "type": "12 fixed radix-2 stages"
          },
          {
            "field": "stages[i].roots_u32",
            "pair_order": "j=0..stage_length/2-1",
            "shape": "[stage_length/2][2]",
            "total_complex_pairs": 4095,
            "total_u32_words": 8190,
            "type": "U32 raw F32 bit pairs (cos,sin)"
          }
        ],
        "schema_id": "feature-fft4096-math-parameters.v1",
        "schema_id_version_annotation": {
          "provider_json_field": false,
          "source": "schema_id suffix",
          "value": "v1"
        },
        "sha256": "aea85e4fb5dcb0354ac11790466d324f1020a8bf5db3f9d981bc46755751fb33",
        "top_level_fields": [
          "schema",
          "route",
          "source",
          "fft_length",
          "stages",
          "real_ADM_new_route_acceptance"
        ],
        "typed_output": "ValidatedFeatureFft4096MathProvider with private typed stages/root pairs and read-only accessors; no serde_json::Value exposure"
      }
    ],
    "provider_test_files": [
      "crates/jocforge-domain/tests/current_features_provider.rs"
    ],
    "receipt_authority": "proposed internal immutable API mapping following P1.4-A/B identity lineage; not a provider JSON field set or native receipt",
    "scope": "identity-first typed validators for the two selected CurrentFeatures math providers; no CurrentFeatures algorithm or product integration",
    "selected_route": "REAL_FFT4096_RADIX2_MATH_V1",
    "status": "LOCALLY_VERIFIED",
    "supplement": {
      "files": [
        {
          "bytes": 1291,
          "file": "API-SCHEMA.json",
          "sha256": "6841baa2f7435529316b16388c3a106fadebab7cff3b8f8ecdf394963cdd27d6"
        },
        {
          "bytes": 3028,
          "file": "FFT-NUMERICAL-QUALITY.json",
          "sha256": "f17440c8ce7534de6baafdee1077c7d7b5d0e8f2fbeaebb30341c018329ccd8d"
        },
        {
          "bytes": 315386,
          "file": "INPUT-ONLY-LITERAL-VECTORS.json",
          "sha256": "1aaf1f5296569960e188b8d74fc8097b5bfb6099b5a1079366e595011c87bdc3"
        },
        {
          "bytes": 14977,
          "file": "SPEC.md",
          "sha256": "1c298e21d81a702f8298dabba3f28e01576c9602729f5746d6665c0becfdeaa5"
        }
      ],
      "identity": "current-features-producer.math-route.v1",
      "manifest_bytes": 1576,
      "manifest_file": "MANIFEST.json",
      "manifest_sha256": "dcff6c7e009ba1cd5e18a03268eecfc2ac1abeceea319f84830978e68b3271ea",
      "research_basis_files_read": false
    },
    "task_id": "P1.4-C-CURRENT-FEATURES-MATH-PROVIDER-CONTRACTS",
    "validator_module_files": [
      "crates/jocforge-domain/src/frontend/current_features_provider.rs"
    ]
  }
}
```

## P1.4-D1 F04 selected component parameter providers

```json
{
  "base_package_identity": {
    "manifest_bytes": 76302,
    "manifest_file": "MANIFEST.json",
    "manifest_fixture": "crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json",
    "manifest_fixture_reused": true,
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "second_manifest_fixture": false
  },
  "boundary_mutations": [
    "provider-id crosswire",
    "Q15 integer/F32 word type confusion",
    "count and shape drift",
    "F32 word length/case drift",
    "selected profile drift",
    "word regeneration or hardcoding",
    "implementation_ready overclaim",
    "early DSP/receiver/QMF implementation"
  ],
  "dependency_scope": {
    "allowed_use": "strict identity-gated manifest/provider JSON parsing and private typed field access only",
    "archive_sha256": "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6",
    "canonical_writer": "none; exact file identity only",
    "crate": "serde_json",
    "default_features": false,
    "features": [
      "std"
    ],
    "public_serde_json_value": false,
    "version": "=1.0.154"
  },
  "fixture_files": [
    "crates/jocforge-domain/tests/fixtures/f04_parameters/window-q15.json",
    "crates/jocforge-domain/tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json"
  ],
  "fixture_inventory": [
    {
      "bytes": 2704,
      "path": "crates/jocforge-domain/tests/fixtures/f04_parameters/window-q15.json",
      "sha256": "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130"
    },
    {
      "bytes": 1077,
      "path": "crates/jocforge-domain/tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json",
      "sha256": "5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5"
    }
  ],
  "fixture_manifest": "crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json",
  "fixture_manifest_reused": true,
  "identity_flow": [
    "recompute complete MANIFEST.json bytes/SHA before parsing",
    "require one exact manifest entry for each provider path",
    "verify observed provider length/SHA before UTF-8 or JSON parsing",
    "validate strict exact schema/shape/range/profile and construct private typed output"
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/window_q15_provider.rs",
    "crates/jocforge-domain/src/spatial/component_parameters_provider.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 19312,
      "path": "crates/jocforge-domain/src/spatial/window_q15_provider.rs",
      "sha256": "3593a00feaf527991bce45289fe113c62a5b0f51e70630f7e0f4d8ca41b22e5c"
    },
    {
      "bytes": 14344,
      "path": "crates/jocforge-domain/src/spatial/component_parameters_provider.rs",
      "sha256": "b4d3c456d02cfb4cafc93e870beeb7fc190a293332bf241c746f96e74672d48e"
    },
    {
      "bytes": 2174,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "697124fac06aa7243049d258ede0634d3b5de721743b1238c049ff2b13752afd"
    }
  ],
  "implementation_prohibitions": [
    "no component DSP or F04 state transition",
    "no receiver/QMF code",
    "no other F04 parameter providers",
    "no regenerating or hardcoding provider values",
    "no second MANIFEST fixture",
    "no projection digest",
    "no public serde_json::Value",
    "no claim that provider validation is DSP PASS",
    "no public or third-party distribution"
  ],
  "implementation_status": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS",
  "mapping_document": "docs/implementation/COMPONENT-PARAMETER-PROVIDER-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "component DSP",
    "receiver/QMF processing",
    "F04 root consumption",
    "render/fanout",
    "parent CallbackTransaction",
    "real ADM execution",
    "product/listening acceptance"
  ],
  "outputs": {
    "component_provider": "private typed read-only ComponentParameterProvider; raw F32 bits preserved; no serde_json::Value exposure",
    "component_receipt": {
      "fields": [
        "validated_manifest_lineage",
        "provider_file_sha256",
        "provider_file_bytes",
        "schema",
        "route",
        "selected_profile",
        "asset_implementation_ready",
        "authorized_scope",
        "dsp_consumption_status"
      ],
      "qualification": {
        "asset_implementation_ready": false,
        "authorized_scope": "INTERNAL_SELECTED_PROFILE_PROVIDER_VALIDATION_AND_FUTURE_CONSUMPTION",
        "dsp_consumption_status": "NOT_YET_VALIDATED"
      },
      "type": "ComponentParameterProviderReceipt"
    },
    "receipt_authority": "immutable identity/typed-validation receipt only; receipt does not imply DSP PASS",
    "window_provider": "private typed read-only WindowQ15Provider",
    "window_receipt": {
      "fields": [
        "validated_manifest_lineage",
        "provider_file_sha256",
        "provider_file_bytes",
        "schema",
        "parameter_ids"
      ],
      "type": "WindowQ15ProviderReceipt"
    }
  },
  "permissions": {
    "delivery_scope": "INTERNAL_RESEARCH_AND_INTERNAL_IMPLEMENTATION_ONLY",
    "private_vendor_key_included": false,
    "public_or_third_party_distribution": "NOT_AUTHORIZED",
    "source_table_copied": false
  },
  "provider_contracts": [
    {
      "bytes": 2704,
      "canonical_preimage": "NOT_PRESENT; identity only; no projection digest",
      "kind": "GENERATED",
      "metadata": {
        "private_vendor_key_included": false,
        "product_role": "ALGORITHM_PARAMETER_NOT_SONG_LOOKUP"
      },
      "metadata_fields": {
        "all_words_research_comparison": "PASS",
        "origin": "MATHEMATICAL_FORMULA_GENERATED_THIS_TASK",
        "raw_asset_or_source_code_copied": false
      },
      "path": "parameters/window-q15.json",
      "profile_binding": "N-bound only; this provider file does not declare an independent sample-rate binding",
      "provider_id": "F04_WINDOW_Q15",
      "record_contracts": [
        {
          "F32LE_sha256": "3968b2666444f0395b008e8d8eb9d84eb2663b4d11199a486b8cbadcf5c142b2",
          "consumer": "receiver double-end blend; validator only",
          "count": 32,
          "generation": "RNE_integer(32768*sin((j+1/2)*pi/(2*N))^2)",
          "parameter_id": "receiver_blend32",
          "q15_values": {
            "json_bool_is_integer": false,
            "range": "0..=32768",
            "shape": [
              32
            ],
            "type": "JSON unsigned integer"
          },
          "representation": "q15_unsigned_integer",
          "to_F32": "R32(q15_value/32768), exact"
        },
        {
          "F32LE_sha256": "19d2e9b70eae909b40027337bda0131132308eabe800cea731ee3e2c1d150b31",
          "consumer": "F04A.8 component filter kernel",
          "count": 64,
          "generation": "RNE_integer(32768*sin((j+1/2)*pi/(2*N))^2)",
          "parameter_id": "component_kernel64",
          "q15_values": {
            "json_bool_is_integer": false,
            "range": "0..=32768",
            "shape": [
              64
            ],
            "type": "JSON unsigned integer"
          },
          "representation": "q15_unsigned_integer",
          "to_F32": "R32(q15_value/32768), exact"
        }
      ],
      "record_fields": [
        "parameter_id",
        "count",
        "representation",
        "q15_values",
        "generation",
        "to_F32",
        "F32LE_sha256",
        "origin",
        "all_words_research_comparison",
        "raw_asset_or_source_code_copied"
      ],
      "schema": "generated-window-parameters.v1",
      "schema_version": "v1",
      "sha256": "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130",
      "top_level_fields": [
        "schema",
        "product_role",
        "private_vendor_key_included",
        "parameters"
      ]
    },
    {
      "bytes": 1077,
      "canonical_preimage": "NOT_PRESENT; identity only; no projection digest",
      "coefficient_rules": [
        "Q15_RNE(2^(-dt/(1/25)))",
        "1-c0",
        "Q15_RNE(2^(-dt/(4/25)))",
        "1-c2",
        "Q15_RNE(2^(-dt/(1/200)))"
      ],
      "dt_rule": "64/48000 seconds",
      "envelope_weight_rule": "Q15_RNE(0.5*10^(3/40))",
      "f32_words": {
        "all_finite": true,
        "coefficient_words_shape": [
          5
        ],
        "preserve_raw_bits": true,
        "ranges": {
          "c0_to_c4": "[0,1]",
          "envelope_weight": "[0,1]",
          "reciprocal_pivot": "[0.5,1)",
          "target_pivot": "[0,1]"
        },
        "scalar_word_count": 3,
        "semantic_order": [
          "c0",
          "c1",
          "c2",
          "c3",
          "c4",
          "reciprocal_pivot",
          "target_pivot",
          "envelope_weight"
        ],
        "type": "lowercase eight-hex-digit raw binary32 word"
      },
      "kind": "GENERATED",
      "metadata": {
        "implementation_ready": "NO",
        "original_design_rationale": "not claimed; equivalent generation settings only",
        "public_or_third_party_distribution": "NOT_AUTHORIZED",
        "source_table_copied": false
      },
      "path": "parameters/component-parameter-mathematical-v1.json",
      "profile_binding": {
        "block_samples": 64,
        "emdf_carriage": "deployed_compat default or etsi_emdf optional",
        "render_target_binding": false,
        "row_count": 15,
        "sample_rate": 48000,
        "selected_profile": "supported ADM BWF / 38-channel PCM24 / 48 kHz; 5.1 six-carrier; DEE mode0/phase1/index3; single independent/zero dependent; 1024 kbps; 6 blocks; 1536 samples/AU; 4096 bytes/AU"
      },
      "provider_id": "F04_COMPONENT_PARAMETER_MATH_V1",
      "q15_integers": {
        "json_bool_is_integer": false,
        "range": "0..=32768",
        "semantic_order": [
          "c0",
          "c1",
          "c2",
          "c3",
          "c4",
          "reciprocal_pivot",
          "target_pivot",
          "envelope_weight"
        ],
        "shape": [
          8
        ],
        "type": "JSON unsigned integer"
      },
      "reciprocal_pivot_rule": "ceil(32768/sqrt(2))/32768",
      "route": "COMPONENT_FIXED_48K_Q15_PARAMETER_MATH_V1",
      "schema": "component-parameter-mathematical-generation.v1",
      "schema_version": "v1",
      "sha256": "5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5",
      "target_pivot_rule": "Q15_RNE(10^(-3/40))",
      "top_level_fields": [
        "schema",
        "route",
        "sample_rate",
        "block_samples",
        "coefficient_rules",
        "dt_rule",
        "reciprocal_pivot_rule",
        "target_pivot_rule",
        "envelope_weight_rule",
        "q15_integers",
        "coefficient_words",
        "reciprocal_pivot_word",
        "target_pivot_word",
        "envelope_weight_word",
        "original_design_rationale",
        "source_table_copied",
        "public_or_third_party_distribution",
        "implementation_ready"
      ]
    }
  ],
  "scope": "identity-first typed validation of the two selected F04 component-route parameter providers only; no DSP, receiver/QMF, rendering, or parent integration",
  "status": "LOCALLY_VERIFIED",
  "task_id": "P1.4-D1-F04-SELECTED-COMPONENT-PARAMETER-PROVIDER-CONTRACTS",
  "test_files": [
    "crates/jocforge-domain/tests/f04_parameter_providers.rs"
  ],
  "test_inventory": [
    {
      "bytes": 6325,
      "path": "crates/jocforge-domain/tests/f04_parameter_providers.rs",
      "sha256": "37d56c997a903e6029828ce41e31c5b27218a1a472a4edf7d53f3fdc38ffeb4b"
    }
  ],
  "test_scope": [
    "manifest and provider identity-first gates",
    "exact top-level and record fields",
    "Q15 integer versus bool/float",
    "F32 raw word syntax/finiteness/ranges",
    "shape, ID uniqueness and provider binding",
    "selected 48 kHz/15-row/64-sample profile",
    "component readiness qualification",
    "malformed JSON rejected only after identity passes"
  ],
  "verification": {
    "DSP": "NOT_IMPLEMENTED",
    "component_provider_count": 1,
    "integration_test_count": 7,
    "module_unit_test_count": 11,
    "provider_validation_is_DSP_PASS": false,
    "window_provider_count": 1
  },
  "wash_side_semantic_clarification": {
    "component_provider_implementation_ready": "NO",
    "date": "2026-10-18",
    "meaning": "component provider implementation_ready=NO disclaims DSP/full-F04/real-ADM/listening/product readiness; it does not prohibit identity-first provider validation or typed provider construction",
    "package_manifest_implementation_ready": "YES",
    "provider_file_modified": false,
    "source": "wash-side authoritative clarification"
  }
}
```

## P1.4-D2 F04 remaining provider validators

```json
{
  "base_package_identity": {
    "manifest_bytes": 76302,
    "manifest_file": "MANIFEST.json",
    "manifest_fixture": "crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json",
    "manifest_fixture_reused": true,
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "second_manifest_fixture": false
  },
  "boundary_mutations": [
    "provider identity/path crosswire",
    "schema or route drift",
    "array/scalar classification and word-type confusion",
    "shape/count/digest drift",
    "generated provider substitution",
    "descriptor qualification overclaim",
    "early FFT/QMF/DSP implementation",
    "public Value or IO leak",
    "second manifest fixture"
  ],
  "dependency_scope": {
    "serde_json": "exact identity-gated provider parsing and private typed field access only",
    "sha2": "manifest and provider file identity only; no provider projection digest"
  },
  "fixture_files": [
    "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json",
    "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json",
    "crates/jocforge-domain/tests/fixtures/f04_parameters/descriptor-mathematical.json"
  ],
  "fixture_inventory": [
    {
      "bytes": 20325,
      "path": "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json",
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c"
    },
    {
      "bytes": 6983,
      "path": "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json",
      "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431"
    },
    {
      "bytes": 4861,
      "path": "crates/jocforge-domain/tests/fixtures/f04_parameters/descriptor-mathematical.json",
      "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197"
    }
  ],
  "fixture_manifest": "crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json",
  "fixture_manifest_reused": true,
  "functional_ids": [
    "F04Q",
    "F04QF",
    "F04R",
    "F04D"
  ],
  "identity_flow": [
    "recompute complete MANIFEST.json bytes/SHA before parsing",
    "require one exact manifest entry for each provider path",
    "verify observed provider length/SHA before UTF-8 or JSON parsing",
    "validate exact schema, route, fields, U32/F32 words, counts, shapes, declared digests, and qualification; construct private typed outputs"
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/qmf_receiver_functional_provider.rs",
    "crates/jocforge-domain/src/spatial/qmf_mathematical_fft_provider.rs",
    "crates/jocforge-domain/src/spatial/descriptor_mathematical_provider.rs",
    "crates/jocforge-domain/src/spatial/window_q15_provider.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 22614,
      "path": "crates/jocforge-domain/src/spatial/qmf_receiver_functional_provider.rs",
      "sha256": "dc04957542f802fa7a751d88911180cef7575c11033fd9a06b5221e7697e4bff"
    },
    {
      "bytes": 11066,
      "path": "crates/jocforge-domain/src/spatial/qmf_mathematical_fft_provider.rs",
      "sha256": "bddb8bd2b775b2cd9fc3e6412981965ef9ca5c5f98d137ff98de31c87dbbb2d8"
    },
    {
      "bytes": 10441,
      "path": "crates/jocforge-domain/src/spatial/descriptor_mathematical_provider.rs",
      "sha256": "df8612dcdc46fcf2620e2a73d68d26d392bbba02be27af729f8c5ded76cf8639"
    },
    {
      "bytes": 19312,
      "path": "crates/jocforge-domain/src/spatial/window_q15_provider.rs",
      "sha256": "3593a00feaf527991bce45289fe113c62a5b0f51e70630f7e0f4d8ca41b22e5c"
    },
    {
      "bytes": 2174,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "697124fac06aa7243049d258ede0634d3b5de721743b1238c049ff2b13752afd"
    }
  ],
  "implementation_prohibitions": [
    "no QMF FFT execution",
    "no receiver or descriptor DSP",
    "no state transition or audio processing",
    "no hardcoded or regenerated provider words",
    "no provider substitution",
    "no second MANIFEST fixture",
    "no projection digest",
    "no public serde_json::Value",
    "no qualification or acceptance overclaim"
  ],
  "implementation_status": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS",
  "mapping_document": "docs/implementation/F04-PROVIDER-VALIDATION-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "FFT execution",
    "QMF receiver and synthesis",
    "descriptor control",
    "F04 state consumption",
    "render/fanout",
    "parent CallbackTransaction",
    "real ADM execution",
    "product/listening acceptance"
  ],
  "outputs": {
    "projection_digest": false,
    "providers": [
      "private read-only QmfReceiverFunctionalProvider",
      "private read-only QmfMathematicalFftProvider",
      "private read-only DescriptorMathematicalProvider"
    ],
    "public_serde_json_value": false,
    "qualification_fields_by_asset": {
      "descriptor_mathematical": [
        "origin",
        "gain_formula",
        "threshold_formula",
        "gain_research_comparison",
        "threshold_research_comparison",
        "threshold_route_status",
        "private_vendor_key_included"
      ],
      "qmf_mathematical_fft": [
        "route",
        "source",
        "audio_acceptance"
      ],
      "qmf_receiver_functional": [
        "delivery_scope",
        "authorization",
        "not_included",
        "array_source_classes",
        "qmf_fft_provider",
        "descriptor_gain_provider",
        "receiver_blend_provider"
      ]
    },
    "receipt_authority": "immutable identity and typed-provider validation only; never DSP, FFT, QMF, descriptor, audio, or product PASS",
    "receipt_fields": [
      "validated_manifest_lineage",
      "provider_file_sha256",
      "provider_file_bytes",
      "schema",
      "asset-specific qualification"
    ]
  },
  "provider_contracts": [
    {
      "arrays": [
        {
          "consumer": "F04Q synthesis prototype; analysis is exact reverse, so no second delivered table",
          "count": 640,
          "f32le_sha256": "870d3c80e4fe7ee368b5da1f7db487dfa84be44c4cf8c82813b78fb06a8e94a2",
          "field": "qwin_synthesis_640",
          "source_class": "PUBLIC_STANDARD_FUNCTIONAL_VALUES_WITH_VERIFIED_EXISTING_COPY"
        },
        {
          "consumer": "F04Q mode=1 audio surround band0 FIR and mode=0 fitting history update contract",
          "count": 21,
          "f32le_sha256": "b352accaefeb0ff0bac1981dcbe9112ff9bd9437e14f1662076b41cf9fd70c70",
          "field": "band0_real_mode1_21",
          "source_class": "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS"
        },
        {
          "consumer": "F04Q mode=1 audio surround band0 FIR; current selected fitting mode=0 does not multiply taps",
          "count": 21,
          "f32le_sha256": "946593788adebf7dd8beafb057c77e167e5b066607306d2a4e31d99244b8b3be",
          "field": "band0_imag_mode1_21",
          "source_class": "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS"
        },
        {
          "consumer": "F04R lane3 four-section pair-order post-filter",
          "count": 20,
          "f32le_sha256": "ddd163cfb2116be81bf283746b3faf38fe771b0d01cee1bf31ea3b0fd04a16eb",
          "field": "lfe_post_filter_4x5",
          "source_class": "AUTHORIZED_NONSECRET_FUNCTIONAL_CONSTANTS_BOUND_TO_FROZEN_BITEXACT_MODEL"
        },
        {
          "consumer": "F04D strict lookup for ordinals 0..126",
          "count": 127,
          "f32le_sha256": "5ba787b1ee814e20bbc86290eca1daa6af33968e4e2458b5a3011c87a78fceac",
          "field": "descriptor_threshold_127",
          "source_class": "AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS"
        }
      ],
      "asset": "qmf_receiver_functional",
      "authorization": "all ordinary non-secret functional constants may be delivered; vendor-secret credentials remain excluded",
      "bytes": 20325,
      "counts": {
        "delivered_array_f32_words": 829,
        "delivered_scalar_f32_words": 12,
        "excluded_unused_mode_other_imag_words": 21
      },
      "delivery_scope": "internal isolated implementation only; no public or third-party redistribution",
      "fixture": "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-receiver-functional-v1.json",
      "functional_ids": [
        "F04Q",
        "F04R",
        "F04D"
      ],
      "generated_providers_reused": {
        "QMF_FFT_RADIX2_MATH_V1": "parameters/qmf-mathematical-fft-v1.json",
        "descriptor_gain_PG03": "parameters/descriptor-mathematical.json",
        "receiver_blend32_PG01": "parameters/window-q15.json"
      },
      "not_included": [
        "vendor-secret credentials",
        "authentication secrets",
        "research source",
        "raw original table artifacts",
        "compiled images",
        "addresses"
      ],
      "path": "parameters/qmf-receiver-functional-v1.json",
      "qualification": {
        "authorization_and_delivery_scope_preserved": true,
        "consumer_validation": "NOT_RUN",
        "provider_validation_only": true
      },
      "scalar_u32": {
        "count": 12,
        "fields": [
          "preconditioning_alpha",
          "preconditioning_beta",
          "preconditioning_tiny_strict",
          "accumulator_magnitude_threshold",
          "accumulator_delta_threshold",
          "accumulator_smoothing",
          "descriptor_level_offset",
          "descriptor_level_linear",
          "descriptor_level_quadratic",
          "descriptor_headroom_step",
          "descriptor_result_scale",
          "descriptor_rise"
        ],
        "type": "JSON U32 raw finite F32 bits"
      },
      "schema": "qmf-receiver-authorized-functional-parameters-v1",
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c",
      "spec_files": [
        "specs/04-qmf-receiver-whole-binding.md",
        "specs/04-qmf-prototype-source-and-loading.md",
        "specs/04-descriptor-control.md"
      ],
      "top_level_fields": [
        "schema",
        "delivery_scope",
        "authorization",
        "not_included",
        "arrays",
        "scalar_u32",
        "generated_providers_reused",
        "counts"
      ]
    },
    {
      "asset": "qmf_mathematical_fft",
      "audio_acceptance": "NOT_RUN",
      "bytes": 6983,
      "fixture": "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json",
      "functional_ids": [
        "F04QF",
        "F04Q"
      ],
      "path": "parameters/qmf-mathematical-fft-v1.json",
      "qualification": {
        "audio_acceptance": "NOT_RUN",
        "fft_execution": "NOT_RUN",
        "provider_validation_only": true
      },
      "root_word_shape": [
        2
      ],
      "rotation_word_count": 128,
      "route": "QMF_FFT_RADIX2_MATH_V1",
      "schema": "qmf-mathematical-fft-parameters.v1",
      "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431",
      "source": "MATHEMATICAL_GENERATION_NO_COMPILED_VALUES_USED",
      "spec_files": [
        "specs/04-qmf-mathematical-fft.md",
        "specs/04-qmf-receiver-whole-binding.md"
      ],
      "stage_fields": [
        "length",
        "roots_u32"
      ],
      "stage_lengths": [
        2,
        4,
        8,
        16,
        32,
        64
      ],
      "stage_root_counts": [
        1,
        2,
        4,
        8,
        16,
        32
      ],
      "top_level_fields": [
        "schema",
        "route",
        "source",
        "stages",
        "rotation_u32",
        "audio_acceptance"
      ],
      "word_type": "JSON U32 raw finite F32 bits"
    },
    {
      "asset": "descriptor_mathematical",
      "bytes": 4861,
      "fixture": "crates/jocforge-domain/tests/fixtures/f04_parameters/descriptor-mathematical.json",
      "functional_ids": [
        "F04D"
      ],
      "gain_formula": "(32+j%32)*2^(floor(j/32))/512",
      "gain_range": "positive finite F32",
      "gain_research_comparison": "PASS_ALL128_WORDS",
      "gain_word_count": 128,
      "origin": "MATHEMATICAL_FORMULAS_GENERATED_IN_THIS_TASK",
      "path": "parameters/descriptor-mathematical.json",
      "private_vendor_key_included": false,
      "qualification": {
        "gain_research_comparison": "PASS_ALL128_WORDS",
        "provider_validation_only": true,
        "threshold_research_comparison": "36_WORD_DIFFERENCES_NOT_IDENTICAL",
        "threshold_route_status": "MATHEMATICAL_CANDIDATE_AUDIO_ACCEPTANCE_PENDING"
      },
      "schema": "descriptor-mathematical-parameters.v1",
      "sha256": "a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197",
      "spec_files": [
        "specs/04-descriptor-control.md",
        "specs/04-qmf-receiver-whole-binding.md"
      ],
      "threshold_formula": "R32(-log2(16*gain[j])/8), j0..126",
      "threshold_research_comparison": "36_WORD_DIFFERENCES_NOT_IDENTICAL",
      "threshold_route_status": "MATHEMATICAL_CANDIDATE_AUDIO_ACCEPTANCE_PENDING",
      "threshold_word_count": 127,
      "top_level_fields": [
        "schema",
        "origin",
        "gain_formula",
        "gain_u32",
        "threshold_formula",
        "threshold_u32",
        "gain_research_comparison",
        "threshold_research_comparison",
        "threshold_route_status",
        "private_vendor_key_included"
      ],
      "word_type": "JSON U32 raw finite F32 bits"
    }
  ],
  "scope": "identity-first typed validation of the QMF receiver functional, QMF mathematical FFT, and descriptor mathematical providers only; no consumer algorithms, DSP, audio, state transition, render, or parent transaction",
  "status": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS",
  "task_id": "P1.4-D2-F04-REMAINING-PROVIDER-VALIDATORS",
  "test_files": [
    "crates/jocforge-domain/tests/f04_remaining_provider_validators.rs"
  ],
  "test_inventory": [
    {
      "bytes": 9771,
      "path": "crates/jocforge-domain/tests/f04_remaining_provider_validators.rs",
      "sha256": "8c0890d03b626bc75bc6daee4150c06d207cba02500778cfbd9288f44ad706a8"
    }
  ],
  "test_scope": [
    "manifest and per-provider identity before parsing",
    "unique exact manifest entries",
    "exact schemas and fields",
    "array/scalar U32 classification and finite F32 bits",
    "counts, stage lengths, root pairs, rotation, and F32LE digests",
    "generated-provider binding without substitution",
    "descriptor qualification preservation",
    "no public Value, IO, DSP, FFT, QMF, descriptor, audio, or render implementation"
  ],
  "verification": {
    "consumer_algorithms": "NOT_IMPLEMENTED",
    "descriptor_mathematical_provider_count": 1,
    "integration_test_count": 7,
    "provider_module_unit_test_count": 12,
    "provider_validation_is_consumer_PASS": false,
    "qmf_mathematical_fft_provider_count": 1,
    "qmf_receiver_functional_provider_count": 1,
    "status": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS"
  }
}
```

## QMF FFT primitive mapping

```json
{
  "aggregate_readiness_unchanged": {
    "P1.3-E": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
    "spatial_audio_root": "MAPPING_COMPLETE_BLOCKED"
  },
  "analysis_synthesis_boundary": {
    "analysis_input_preparation": "the caller computes R32(x/64) independently for all 128 scalar inputs before constructing QmfFftInput; this scaling is not part of execute_qmf_fft64",
    "primitive_api_boundary": "the fixed-size primitive accepts one validated QmfFftInput and QmfMathematicalFftProvider; it has no mode, direction, normalization, or caller-reorder argument",
    "synthesis_input_preparation": "the caller passes all 128 scalar inputs unchanged before constructing QmfFftInput; no scaling is part of execute_qmf_fft64"
  },
  "caller_boundaries": {
    "analysis_after_fft": "for p=0..31: out[4p]=U[2p]; out[4p+1]=U[2p+1] XOR 0x80000000; out[4p+2]=U[126-2p]; out[4p+3]=U[127-2p]",
    "analysis_before_fft": "F04Q.2 polyphase fold then F04Q.3 complex rotation; neither operation is part of the FFT primitive",
    "rotation_provider_boundary": "rotation_u32 is a separate 128-word QMF caller parameter from the same provider; the FFT primitive consumes stages[].roots_u32 only and does not infer rotation indexing from root pairs",
    "synthesis_after_fft": "for FFT output (r,i) and rotation (a,b): real=((r*b)+(i*a))*2; imag=((i*b)-(r*a))*2 with each R32 operation separate; F04Q.6 polyphase output doubling is separate and must not be fused",
    "synthesis_before_fft": "for p=0..31: V[2p]=U[4p]; V[2p+1]=U[4p+1] XOR 0x80000000; V[126-2p]=U[4p+2]; V[127-2p]=U[4p+3]; this is the inverse caller permutation"
  },
  "consumers": {
    "fft_primitive_state": "none; all analysis, delay, FIR, synthesis, and fitting histories belong to their caller owners",
    "fitting_mode0": {
      "carrier_order": [
        "L",
        "R",
        "C",
        "Ls",
        "Rs"
      ],
      "common_path": "same QMF analysis route and roots; independent fitting owner",
      "mode_difference": "mode=0 updates its own delay/FIR history, does not run the 21-tap multiply, does not overwrite band0, and has no synthesis FFT",
      "receiver_lanes": [
        0,
        1,
        2,
        4,
        5
      ],
      "spec": "F04RB.2/F04RB.3",
      "state_owner": "spatial_audio QmfWindowState.fitting_delay and fitting_fir; distinct from receiver mode=1 histories"
    },
    "receiver_mode1": {
      "common_path": "shared analysis FFT then F04Q.4 mode=1 delay/sign rotation and band0 FIR; these two lanes then call the same FFT route for synthesis without 1/64 input scaling",
      "spec": "F04RB.2",
      "state_owner": "spatial_audio ModelState.analysis_history, qmf_delay_history, qmf_fir_history, and synthesis_history",
      "surround_lanes": [
        4,
        5
      ]
    },
    "shared_analysis": {
      "fitting_reuse": "carrier fitting recomputes raw analysis from receiver analysis before-state and the same s09_input; the five analysis-after histories must compare wordwise equal, but receiver history is not committed twice",
      "input": "same-generation receiver s09_input after F04R stage 8; each lane has a separate analysis history",
      "receiver_lanes": [
        0,
        1,
        2,
        4,
        5
      ],
      "slots_per_callback": 24,
      "spec": "F04Q"
    }
  },
  "error_contract": {
    "native_error_claim": false,
    "nonfinite_intermediate_or_output": "reject as a fatal numeric error per N00.2; do not clamp or continue",
    "state_effect": "the primitive is stateless and has no commit or retirement authority; callers apply the enclosing F04/I03 outcome",
    "wrong_length_or_nonfinite_input": "reject before transform per F04QF.1/I03; no output"
  },
  "fixture_files": [],
  "fresh_review": {
    "conclusion": "QMF numeric/provider/vector/state identities unchanged; current carrier routing is fresh-static only; J05 cross-stage execution remains NOT_READY",
    "execution_or_product_pass": false,
    "manifest_sha256": "75e7313563c37f79a9afb895704b5eb77e6dda55d41117f820e62c445bbe4999",
    "old_embedded_review_records": "STALE_IDENTITY_DO_NOT_USE_AS_CURRENT_PASS",
    "review_sha256": "926c8a3c9cd21e68665ac287ad98403481b55adf3cdeef6db7660c0f4fb37a60",
    "static_checks": "44/44",
    "status_sha256": "4aa0f1d9f0fbc718772193159069077618e0c0138c8002798c893d29f0296fb8",
    "supplement_directory": "outputs/IMPLEMENTER_SUPPLEMENTS/F04-CURRENT-FRESH-REVIEW-R1"
  },
  "functional_id": "F04QF",
  "functional_id_scope": "This readiness applies only to the fixed 64-point FFT primitive. F04QF rotation use, F04Q analysis/synthesis, receiver, fitting, and whole spatial_audio remain separately scoped.",
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/qmf_fft64.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 3812,
      "path": "crates/jocforge-domain/src/spatial/qmf_fft64.rs",
      "sha256": "d393d09d14346d9c275650ea2918b6e59a18d5113a82d1a9208d9ed044923f18"
    },
    {
      "bytes": 2174,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "697124fac06aa7243049d258ede0634d3b5de721743b1238c049ff2b13752afd"
    }
  ],
  "implementation_prohibitions": [
    "no generic FFT",
    "no F04Q fold or rotation implementation",
    "no receiver/QMF state transition",
    "no descriptor, fitting, or DSP implementation",
    "no CurrentFeatures 4096-point FFT substitution",
    "no J05 cross-stage execution or readiness promotion",
    "no product/audio/listening acceptance claim"
  ],
  "implementation_status": "LOCALLY_VERIFIED_FIXED64_ONLY",
  "mapping_document": "docs/implementation/QMF-FFT-PRIMITIVE-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "module_export_file": "crates/jocforge-domain/src/spatial/mod.rs",
  "package_identity": {
    "manifest_bytes": 76302,
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
  },
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/qmf_fft64.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/tests/qmf_fft64.rs"
  ],
  "planned_vector_sources": [
    "specs/04-qmf-mathematical-fft.md",
    "test-vectors/QMF-MATHEMATICAL-FFT-VERIFICATION.json",
    "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json"
  ],
  "primitive_contract": {
    "arithmetic_semantics": "each multiply/add/subtract is a separate R32 F32 rounding point; no FMA; do not regroup products or layer sums",
    "butterfly_order": [
      "tr=(c*bre) - (d*bim)",
      "ti=(c*bim) + (d*bre)",
      "out_a=(a_re+tr,a_im+ti)",
      "out_b=(a_re-tr,a_im-ti)"
    ],
    "complex_point_count": 64,
    "exponent": "negative-exponent DFT; caller mode and scaling are outside this primitive",
    "input_bit_reversal": "rev6(k) reverses the six bits of k; work[k] copies input[rev6(k)] without swapping the real/imag pair",
    "input_layout": "[real0, imag0, real1, imag1, ..., real63, imag63]; finite F32; each real/imag pair stays in that order",
    "input_validation": "require exactly 128 finite F32 scalar slots; wrong length or nonfinite input rejects per F04QF.1/I03 before output",
    "output_layout": "64 complex values in natural frequency order k=0..63, interleaved [real0,imag0,...,real63,imag63]",
    "persistent_state": "none; a slot's work buffer is initialized from its own input and never seeded from a prior slot",
    "real_scalar_slot_count": 128,
    "root_formula": "c=R32(cos(-2*pi*j/L)); d=R32(sin(-2*pi*j/L)); roots are indexed by j within each stage",
    "root_provider_shape": "six stage records with lengths [2,4,8,16,32,64], root counts [1,2,4,8,16,32], each root pair [real_u32,imag_u32]; 63 complex roots total",
    "root_special_cases": "L=2 uses (1,+0); for L>2 and j a multiple of L/4, use exact integer quadrant pairs (1,+0),(+0,-1),(-1,+0),(+0,1) selected by quadrant modulo four",
    "rounding_semantics": "R32 is IEEE-754 binary32 round-to-nearest ties-to-even per N00.1; preserve signed zero and gradual underflow per N00.2",
    "route": "QMF_FFT_RADIX2_MATH_V1",
    "runtime_root_source": "validated provider U32 words; do not call platform libm or regenerate roots at runtime",
    "stage_iteration": "process L in listed order; base=0,L,2L,...,64-L; within each group j=0..L/2-1; read both a and b from the current stage values before either destination is written",
    "stage_lengths": [
      2,
      4,
      8,
      16,
      32,
      64
    ]
  },
  "programme_root": "spatial_audio",
  "provider_binding": {
    "consumed_fields": [
      "stages[].length",
      "stages[].roots_u32[][0..1] as [real_u32,imag_u32]"
    ],
    "not_consumed_by_primitive": [
      "rotation_u32",
      "audio_acceptance"
    ],
    "provider_bytes": 6983,
    "provider_file": "parameters/qmf-mathematical-fft-v1.json",
    "provider_qualification": "identity and typed shape validation is LOCALLY_VERIFIED; fixed64 primitive execution is LOCALLY_VERIFIED_FIXED64_ONLY; no QMF caller is implemented",
    "provider_schema": "qmf-mathematical-fft-parameters.v1",
    "provider_sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431",
    "provider_source": "crates/jocforge-domain/src/spatial/qmf_mathematical_fft_provider.rs",
    "provider_validation_slice": "P1.4-D2",
    "root_values_source": "the pinned provider file carries exact U32 root words; consume typed stages[].roots without regenerating values",
    "typed_provider": "QmfMathematicalFftProvider"
  },
  "scope": "Implement and locally verify only the fixed 64-point QMF radix-2 FFT primitive. Do not implement QMF fold/rotation, receiver, descriptor, fitting, state transition, or DSP.",
  "shared_fixture_dependencies": [
    "crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json",
    "crates/jocforge-domain/tests/fixtures/f04_parameters/qmf-mathematical-fft-v1.json"
  ],
  "specifications": [
    {
      "bytes": 3369,
      "clauses": [
        "N00.1",
        "N00.2"
      ],
      "file": "specs/00-numeric-semantics.md",
      "sha256": "53cb67d6c9ddced1fb909dda806bb478a2926a66f2e38cecaada6f6e95d4d135"
    },
    {
      "bytes": 3258,
      "clauses": [
        "F04QF.1",
        "F04QF.2",
        "F04QF.4"
      ],
      "file": "specs/04-qmf-mathematical-fft.md",
      "sha256": "e915152a13098a18c4028ef5f3e72ba417341cdf87b17dc76645e73395319823"
    },
    {
      "bytes": 5584,
      "clauses": [
        "F04Q.1",
        "F04Q.3",
        "F04Q.5",
        "F04Q.7"
      ],
      "file": "specs/04-qmf-analysis-and-synthesis.md",
      "sha256": "13a2907c8e60bd81f948f33b9ad07f2d7dc0552b0b12f53224d27b80af610edf"
    },
    {
      "bytes": 4428,
      "clauses": [
        "F04QP.1",
        "F04QP.3"
      ],
      "file": "specs/04-qmf-prototype-source-and-loading.md",
      "sha256": "8255e0d95eb91db0c5ecf6b120bab4b7f60796d121bf9249bb51db72bc91811f"
    },
    {
      "bytes": 13595,
      "clauses": [
        "F04RB.1",
        "F04RB.2",
        "F04RB.8",
        "F04RB.9"
      ],
      "file": "specs/04-qmf-receiver-whole-binding.md",
      "sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75"
    }
  ],
  "status": "LOCALLY_VERIFIED_FIXED64_ONLY",
  "task_id": "QMF-FFT-PRIMITIVE-MAPPING",
  "test_files": [
    "crates/jocforge-domain/tests/qmf_fft64.rs"
  ],
  "test_inventory": [
    {
      "bytes": 8575,
      "path": "crates/jocforge-domain/tests/qmf_fft64.rs",
      "sha256": "9d53ca5c441352312dfc19e425e990145e0416e3bcb0d733d2642c38fb119014"
    }
  ],
  "vector_bindings": [
    {
      "bytes": 1158,
      "file": "test-vectors/QMF-MATHEMATICAL-FFT-VERIFICATION.json",
      "qualification": "synthetic mathematical evidence only; production audio validation NOT_RUN",
      "scope": "160 synthetic cases per mode; 128 basis cases and 20480 values per mode; report records analysis/synthesis numeric comparison and selected route",
      "sha256": "02d93b89be10293f37ae283b80efcca5dfd434d4fd31b0494855e420db004f2f"
    },
    {
      "bytes": 74454,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "qualification": "model-first consumer vector; current fresh review is static only and cross-stage J05 execution remains NOT_READY",
      "scope": "five continuous input-only callbacks from cold using QMF_FFT_RADIX2_MATH_V1; inputs are deterministic recipes and expected hashes are assertions, never seeds",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    {
      "bytes": 1468,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-RESULT.json",
      "qualification": "source package result is not local isolated implementation execution or product acceptance",
      "scope": "scoped whole-binding model-first result and output counts",
      "sha256": "cb42de15ce4fa787aa03e511b9f2c6da39a72d3927f00eb020295af8a02862c7"
    }
  ],
  "verification": {
    "fft_execution": "LOCALLY_VERIFIED_FIXED64_ONLY",
    "fresh_review_scope": "STATIC_ONLY",
    "full_repository_boundary_mutations_rejected": 728,
    "local_test_vector": "spec-derived dyadic input recipe with exact R32 operation order and the identity-validated provider roots; assertion only, not an audio or product vector",
    "product_audio_acceptance": "NOT_RUN",
    "qmf_fft64_test_count": 10,
    "receiver_fitting_execution": "NOT_RUN",
    "spec_provider_vector_identities": "PASS",
    "targeted_boundary_mutations_rejected": 43,
    "typed_provider_validator": "LOCALLY_VERIFIED_TYPED_PROVIDER_VALIDATORS"
  }
}
```

## PostWindow fanout foundation

```json
{
  "boundary": "INSTALLED48_POST_WINDOW_48X3072",
  "current_features_input": {
    "algorithm_changed": false,
    "callback_phase_input": "explicit CallbackPhase; never inferred from sample values",
    "lineage_validation": [
      "programme",
      "attempt",
      "transaction",
      "callback",
      "phase",
      "profile",
      "provider lineage",
      "spatial_control before generation/digest"
    ],
    "local_commit": false,
    "none_behavior": "Allowed only when every source flag is RESET or CARRY; if any source is READ (00), reject with ReadAudioMissing before state or output proposal",
    "some_behavior": "Always validate phase, programme, attempt, transaction, callback, profile, provider lineage, and spatial_control before generation/digest, including when no source reads samples",
    "type": "Option<CurrentFeaturesAudioView>",
    "unused_view_behavior": "The parent may drop an unconsumed CurrentFeaturesAudioView when no source is READ; dropping the view does not commit owner state"
  },
  "fanout": {
    "operation": "fanout(self)",
    "returns_exactly": [
      "CurrentFeaturesAudioView",
      "SpatialAudioPostWindowView"
    ],
    "shared_payload": "both views use the proposal's same immutable Arc allocation; only the Arc handle is cloned",
    "views_clone_copy": false
  },
  "future_spatial_audio_assembler": {
    "additional_same_callback_inputs": [
      "matrix",
      "destinations",
      "LFE inputs"
    ],
    "implemented": false,
    "input_view": "SpatialAudioPostWindowView"
  },
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/post_window_fanout.rs",
    "crates/jocforge-domain/src/spatial/current_features.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "implementation_status": "LOCALLY_VERIFIED__RENDER_PRODUCER_DEFERRED",
  "mapping_document": "docs/implementation/POSTWINDOW-FANOUT-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "render production",
    "matrix/destination production",
    "SpatialAudioInputPacket assembler",
    "F04 DSP",
    "parent transaction",
    "real ADM/audio/runtime/listening acceptance"
  ],
  "phase": [
    "NORMAL",
    "CACHED_DRAIN"
  ],
  "proposal": {
    "clone_copy": false,
    "lineage": [
      "programme",
      "attempt",
      "CallbackTransactionId",
      "FrontendCallbackIndex",
      "CallbackPhase",
      "selected EncodingProfile",
      "validated F01E/F03 provider lineage",
      "spatial_control before generation and complete root digest",
      "typed render-stage identity",
      "typed spatial-control after-proposal identity",
      "payload byte count and SHA-256"
    ],
    "payload_byte_count": 589824,
    "payload_sha256": "SHA-256 over exact source-major F32 U32LE bytes",
    "payload_type": "Arc<[FiniteF32]>",
    "production_constructor": "future spatial_control/F03W renderer only; requires opaque SpatialControlRenderAuthority minted only inside spatial_control; no public raw/default constructor",
    "shape": [
      48,
      3072
    ],
    "source_major": true,
    "type": "linear PostWindowRenderProposal"
  },
  "status": "LOCALLY_VERIFIED__RENDER_PRODUCER_DEFERRED",
  "status_non_promotion": {
    "CURRENT_FEATURES_PRODUCER": "LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED",
    "P1.3-E": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
    "P1.3-E2": "LOCALLY_VERIFIED_COMPONENT_PRODUCER__UPSTREAM_AND_PARENT_DEFERRED"
  },
  "supplement": {
    "api_schema_bytes": 732,
    "api_schema_file": "API-SCHEMA.json",
    "api_schema_sha256": "1ebf6145695ff88013345b9d68aec3266854d8f482351a10c40ded0a3a5134a1",
    "contract_bytes": 2835,
    "contract_file": "CONTRACT.md",
    "contract_sha256": "b04df4c2badee8cb189c5361ab5a42e248af071e1c92109913a733e1f2a4c729",
    "identity": "postwindow-fanout-contract.v1",
    "manifest_sha256": "2a55e66314d6f2aa37cf792b62068185358b1a4b294e6c7b95e8b8d77d99f107",
    "root": "outputs/IMPLEMENTER_SUPPLEMENTS/POSTWINDOW-FANOUT-CONTRACT-V1"
  },
  "task_id": "POSTWINDOW-FANOUT-FOUNDATION-V1",
  "test_files": [
    "crates/jocforge-domain/src/spatial/post_window_fanout/tests.rs",
    "crates/jocforge-domain/src/spatial/current_features/tests.rs"
  ],
  "test_scope": [
    "fixed shape and finite sample admission",
    "source-major payload byte count and SHA-256",
    "explicit normal/cached-drain phase",
    "one-time two-view fanout",
    "shared Arc payload allocation identity",
    "different proposal identities despite equal payload hashes",
    "lineage and view mismatch",
    "consumer error drops capabilities without state commit",
    "capabilities remain non-Clone/non-Copy"
  ],
  "verification": {
    "compile_fail_contracts": 5,
    "current_features_tests": 9,
    "full_repository_boundary_mutations_rejected": 585,
    "matrix_destination_producer": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "post_window_fanout_tests": 5,
    "render_producer": "NOT_IMPLEMENTED",
    "spatial_audio_assembler": "NOT_IMPLEMENTED",
    "targeted_boundary_mutations_rejected": 32
  }
}
```

## P1.3-E3 Direct-LFE ring stage

```json
{
  "clock_contract": {
    "callback_advance": "each successful actual NORMAL or CACHED_DRAIN callback advances the ring once and adds exactly 1536 model samples",
    "downstream_clocks": "receiver r=g-2, receiver after generation g-1, JOC dispatch d=g-1; E3 does not execute or advance receiver/QMF/JOC",
    "full_profile_counts": {
      "actual_callbacks": 4251,
      "cached_drains": 4,
      "normal_callbacks": 4247
    },
    "isolated_vector_trace": "the six supplement cases g0..g5 explicitly label g0/g1 NORMAL and g2..g5 CACHED_DRAIN for a bounded ring trace only; this does not override production phase ranges or imply a second profile clock",
    "output_origin": "for g>=2, row0 origin is source generation g-2",
    "priming": "g=0 and g=1 produce no LFE output row",
    "production_cached_drain": "g=4247..4250",
    "production_domain": "FrontendCallbackIndex newtype supplied by the parent; phase is explicit and never inferred from the numeric generation",
    "production_normal": "g=0..4246",
    "source_origin": "s=g-2 only for g>=2",
    "type10": "for g>=2, Type10 row0 is LFE[g-2], rows1..15 are component15[g] from P1.3-E2; E3 emits row0 input only and does not assemble Type10"
  },
  "commit_authority": "enclosing CallbackTransactionCoordinator root replacement only; no F04C/local E3 commit",
  "eof_drain": "process only actual callbacks, including the four production cached drains; stop after the fourth drain; no synthetic LFE or QMF flush",
  "error_translation": {
    "authority": "WASH_APPROVED_LOCAL_TRANSLATION_2026-10-18; no native/package error-code claim",
    "failure_result": "first applicable local outcome only; no proposal; committed state unchanged",
    "outcomes": [
      {
        "error_code": "F04_PIPELINE_RETIRED",
        "trigger": "retired owner/programme"
      },
      {
        "error_code": "F04_PROFILE_UNSUPPORTED",
        "trigger": "unsupported or mismatched selected profile"
      },
      {
        "error_code": "F04_CALLBACK_ORDER",
        "trigger": "stale/out-of-order generation or phase mismatch",
        "vector_error_label": "STALE_OR_OUT_OF_ORDER_GENERATION"
      },
      {
        "error_code": "F04_STATE_ERROR",
        "trigger": "spatial_audio before-generation/digest mismatch or invalid ring shape/cursor"
      },
      {
        "error_code": "F04_INPUT_ERROR",
        "trigger": "programme/attempt/transaction/provider/semantic-host mismatch or row shape/nonfinite input",
        "vector_error_label": "F04_INPUT_ERROR where supplied"
      }
    ],
    "priority": [
      "retired",
      "selected_profile",
      "callback",
      "owner_generation_digest",
      "other_lineage_and_input"
    ]
  },
  "fatal_behavior": "local validation returns the first mapped error with no proposal; any downstream or parent failure discards the staged LFE proposal with the other staged roots and retires the programme",
  "fixture_files": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json"
  ],
  "fixture_inventory": [
    {
      "bytes": 13306,
      "path": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
      "sha256": "52c2b1a3b52e53198e47e90e4d6e51e8435fbe660ad996b4005f741107a7eb7f"
    },
    {
      "bytes": 639,
      "path": "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json",
      "sha256": "b9d8c32b497c2498e3ae1c2a2822fd5cee2dccd456b5dae68bf3fb5ba0f1f307"
    }
  ],
  "functional_scope": [
    {
      "functional_id": "F04C",
      "status": "SCOPED_SUBSET_ONLY",
      "supplement_sections": [
        "1 ingress and lineage",
        "2 three-slot state machine",
        "3 Type10 row0 projection"
      ]
    }
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 17056,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
      "sha256": "b30cb63addb5f8ddfe6bf03e65f84e34221929b8cb78180efc1e6c762c6b6bc5"
    },
    {
      "bytes": 35171,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "6e3154571999492fdf2f881147ee9bb1a111eb191bd9cba9aaace91075840e03"
    },
    {
      "bytes": 26347,
      "path": "crates/jocforge-domain/src/spatial/spatial_control.rs",
      "sha256": "3c13f67f84698b32d568bf9fda223878eb3925a95c7fcbe30e0d0ef2f1b8fcd2"
    }
  ],
  "implementation_prohibitions": [
    "no raw38-to-LFE producer",
    "no hardcoded LFE host slot",
    "no regular-source mute transform",
    "no component producer or component-row assembly",
    "no Type10 full assembly",
    "no receiver/QMF or JOC/codec logic",
    "no synthetic EOF flush",
    "no local commit/abort/fatal authority",
    "no parent transaction",
    "no F04 full-chain/product acceptance promotion"
  ],
  "implementation_status": "LOCALLY_VERIFIED_DIRECT_LFE__UPSTREAM_PARENT_DEFERRED",
  "input_contract": {
    "cached_drain": "reuse the exact last NORMAL padded LFE row bytes with a new callback generation; do not reread ADM or regenerate samples",
    "host_slot": "typed host identity in 0..37; never hardcode a source/track slot",
    "lineage": [
      "programme",
      "attempt",
      "CallbackTransactionId",
      "FrontendCallbackIndex",
      "explicit CallbackPhase",
      "selected profile",
      "spatial_control before-generation and full digest",
      "provider lineage",
      "unique semantic LFE host identity"
    ],
    "mute": "no mute input; preserve explicit row bytes unchanged; regular-source mute does not enter this path",
    "normal_short_read": "upstream pads the full host38 frame with positive zero before this view is created; E3 receives the full padded row",
    "payload": "owned finite F32[1536], exact F32LE row bytes of 6144 bytes from host38[g][lfe_host][0..1535]",
    "producer_authority": "future spatial_control/F03W producer after validated ADM graph establishes exactly one semantic_role=lfe and bed_id=3 host slot",
    "type": "opaque DirectLfeInputView"
  },
  "mapping_document": "docs/implementation/DIRECT-LFE-RING-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "real ADM read",
    "production raw38-to-current_lfe sample audit",
    "whole callback transaction/fault injection",
    "receiver/QMF",
    "full song/decode/render/listening"
  ],
  "overall_p1_3_e_status_remains": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
  "owner_and_state": {
    "cold": {
      "input_frame_indexes": [
        null,
        null,
        null
      ],
      "model_samples_consumed": 0,
      "read_index": 1,
      "rows": [
        null,
        null,
        null
      ],
      "write_index": 0
    },
    "exact_fields": [
      "write_index:I64LE",
      "read_index:I64LE",
      "rows:T3<None|B(6144)> according to the approved runtime variants",
      "input_frame_indexes:T3<None|I64LE>",
      "model_samples_consumed:I64LE"
    ],
    "functional_owner": "F04C within the single spatial_audio programme root",
    "sample_count_unit": "model F32 samples; not ADM valid samples, bytes, frames, or output samples",
    "staged_result": "linear LfeStageProposal with complete ring after-state and optional immutable Type10LfeRow0Input; no local committed state or success authority",
    "state_pointer": "#/whole_state_by_boundary/cold/fields/5 (lfe_state)",
    "state_schema_bytes": 226344,
    "state_schema_file": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json",
    "state_schema_sha256": "4fdab9f591821d7c50e7daca17bc98d4a5b0616e9cb45b5937cae47b26ed657c"
  },
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe/tests.rs"
  ],
  "planned_test_vector_fixtures": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
    "crates/jocforge-domain/tests/fixtures/direct_lfe/G1-VECTOR-ERRATUM.json"
  ],
  "profile_binding": "selected EncodingProfile only, validated against the spatial_audio owner; all other profiles remain UNSUPPORTED_PROFILE",
  "programme_root": "spatial_audio",
  "receipt": "no E3-specific committed receipt; a future complete F04C/CallbackTransaction owns selected-j05-owner-receipt.v1 and the global OwnerReceipt<SpatialAudio>",
  "reset_authority": "programme-origin full restart reconstructing the complete cold root; no resume within RETIRED attempt",
  "scope": "Consume one opaque F03W-owned direct-LFE input and stage only the three-slot delay-ring transition plus optional Type10 row0 input. Do not implement the raw38 producer, component rows, receiver/QMF, full F04 assembler, receipt wrapper, local commit, parent transaction, or codec/JOC logic.",
  "stage_authority": "F04C direct-LFE substage only; future F04 composer joins this proposal with the P1.3-E2 component proposal",
  "status": "LOCALLY_VERIFIED_DIRECT_LFE__UPSTREAM_PARENT_DEFERRED",
  "supplement": {
    "api_schema_bytes": 652,
    "api_schema_file": "API-SCHEMA.json",
    "api_schema_sha256": "611bacbcc774b217449387c062af86c87f6fda2c523f5db6d4fa703bc9b3073e",
    "g1_erratum_bytes": 639,
    "g1_erratum_file": "G1-VECTOR-ERRATUM.json",
    "g1_erratum_sha256": "b9d8c32b497c2498e3ae1c2a2822fd5cee2dccd456b5dae68bf3fb5ba0f1f307",
    "identity": "direct-lfe-ring-v1-with-g1-erratum",
    "manifest_sha256": "15da6c0cd0802079c1911e161eacb4cd1e1d4afd3e15a606f3daf1988a5f3595",
    "root": "outputs/IMPLEMENTER_SUPPLEMENTS/DIRECT-LFE-RING-V1",
    "spec_bytes": 6204,
    "spec_file": "SPEC.md",
    "spec_sha256": "44fbbd2bd707ca26488b885ee879c448c759e1227a69a78f3817353c69b2051d",
    "vectors_bytes": 13306,
    "vectors_file": "INPUT-ONLY-VECTORS.json",
    "vectors_sha256": "52c2b1a3b52e53198e47e90e4d6e51e8435fbe660ad996b4005f741107a7eb7f"
  },
  "task_id": "P1.3-E3-F04-DIRECT-LFE-RING",
  "test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe/tests.rs"
  ],
  "test_inventory": [
    {
      "bytes": 19748,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe/tests.rs",
      "sha256": "974767ddea57685ddf6656a5e46308e47b2a92cefdb7e23827230d6c9f831c66"
    }
  ],
  "transition": {
    "failure": "no after proposal and no ring/cursor/index/sample-count advance; enclosing callback retires the programme and same-attempt retry is forbidden",
    "next_read": "(old_read+1)%3",
    "next_write": "(old_write+1)%3",
    "output_read": "read old.rows[next_read] and old.input_frame_indexes[next_read] before writing",
    "preflight_before_change": [
      "retired state",
      "selected profile",
      "callback generation and explicit phase",
      "programme/attempt/transaction/provider/host lineage",
      "spatial_audio before generation and full digest",
      "ring indices/shape",
      "payload exact shape and finite values"
    ],
    "sample_advance": "model_samples_consumed += 1536 exactly once after all preflight passes",
    "write_current": "write current_lfe[g] and typed callback generation g into next_write"
  },
  "vectors": {
    "actual_first_rule": "construct rows from the listed input recipe or exact prior cached row, compute bytes/digests and execute the ring; never seed state from expected values or hashes",
    "assertions": [
      "before/after three row digests and frame-index tuples",
      "write/read indices",
      "model sample count",
      "priming/output availability",
      "output origin and exact row bytes",
      "Type10 row0 and component15 origins",
      "failure state unchanged"
    ],
    "cached_rows": "g2..g5 reuse exact padded g1 bytes; do not rerun the impulse recipe",
    "case_ids": [
      "g0",
      "g1",
      "g2",
      "g3",
      "g4",
      "g5"
    ],
    "expected_values_are_assertions_only": true,
    "failure_case_ids": [
      "stale_generation",
      "short_shape",
      "nonfinite"
    ],
    "file": "INPUT-ONLY-VECTORS.json",
    "g1_erratum": {
      "default_u32": 0,
      "fresh_f32le_bytes": 6144,
      "fresh_sha256": "755447224cb411e85201061e13b79d0b020b08642933c5acb161abc0026624e7",
      "negative_zero_indexes": [],
      "override_index": 270,
      "override_u32": "0x3E000000",
      "positive_zero_padding_range": [
        731,
        1536
      ],
      "sample_count": 1536,
      "valid_samples": 731
    }
  },
  "verification": {
    "direct_lfe_test_count": 9,
    "f03w_raw38_producer": "NOT_IMPLEMENTED",
    "failure_vector_cases": 3,
    "full_repository_boundary_mutations_rejected": 629,
    "full_type10_assembler": "NOT_IMPLEMENTED",
    "local_commit": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "receiver_qmf": "NOT_IMPLEMENTED",
    "success_vector_cases": 6,
    "targeted_implementation_mutations_rejected": 22,
    "targeted_mapping_mutations_rejected": 22
  }
}
```

## P1.3-E4 Type10 assembly boundary

```json
{
  "commit_authority": "enclosing callback transaction only; no E4/local commit",
  "error_translation": {
    "authority": "local translation of the existing F04C.7 error categories; no native error-code claim",
    "failure_result": "first applicable local outcome only; no Type10AssemblyProposal or packet; caller-owned committed root unchanged",
    "outcomes": [
      {
        "error_code": "F04_PROFILE_UNSUPPORTED",
        "trigger": "unsupported or proposal/parent profile mismatch"
      },
      {
        "error_code": "F04_CALLBACK_ORDER",
        "trigger": "callback or explicit phase mismatch"
      },
      {
        "error_code": "F04_STATE_ERROR",
        "trigger": "proposal spatial_audio before generation/digest differs from expected parent"
      },
      {
        "error_code": "F04_INPUT_ERROR",
        "trigger": "programme/attempt/transaction/provider lineage mismatch"
      },
      {
        "error_code": "F04_STATE_ERROR",
        "trigger": "missing row0 after priming, row0 origin/ring disagreement, malformed component or LFE output shape, nonfinite row bits"
      }
    ],
    "priority": [
      "selected_profile",
      "callback_phase",
      "spatial_audio_before_generation_digest",
      "cross_proposal_lineage_and_provider",
      "ring_origin_and_payload_shape"
    ],
    "retirement_boundary": "parent/E2/E3 stages reject RETIRED owners before proposals exist; E4 has no programme retirement authority"
  },
  "fatal_behavior": "return the first local error without a packet or stage proposal; parent owns programme retirement",
  "fixture_files": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json"
  ],
  "fixture_inventory": [
    {
      "bytes": 13306,
      "path": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json",
      "sha256": "52c2b1a3b52e53198e47e90e4d6e51e8435fbe660ad996b4005f741107a7eb7f"
    }
  ],
  "functional_scope": [
    {
      "clauses": [
        "F04C.3",
        "F04C.7"
      ],
      "functional_id": "F04C",
      "status": "SCOPED_SUBSET_ONLY"
    }
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 12144,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
      "sha256": "9473ea603741d59955d90f4a58c6c02875f1bba97d45203d6547c9b0c84d8a78"
    },
    {
      "bytes": 35171,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
      "sha256": "6e3154571999492fdf2f881147ee9bb1a111eb191bd9cba9aaace91075840e03"
    },
    {
      "bytes": 38627,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
      "sha256": "ff0121d7a2ad16dac2682ae3564b2d4baafff64b622673a05015133253646256"
    },
    {
      "bytes": 17056,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
      "sha256": "b30cb63addb5f8ddfe6bf03e65f84e34221929b8cb78180efc1e6c762c6b6bc5"
    },
    {
      "bytes": 26347,
      "path": "crates/jocforge-domain/src/spatial/spatial_control.rs",
      "sha256": "3c13f67f84698b32d568bf9fda223878eb3925a95c7fcbe30e0d0ef2f1b8fcd2"
    }
  ],
  "implementation_prohibitions": [
    "no receiver or QMF implementation",
    "no persistent Type10 packet field",
    "no packet Clone/Copy/default/raw constructor",
    "no local commit/abort/fatal authority",
    "no parent CallbackTransaction",
    "no generic row assembler",
    "no JOC/codec logic",
    "no whole-chain/product acceptance promotion"
  ],
  "implementation_status": "LOCALLY_VERIFIED_TYPE10_ASSEMBLY__RECEIVER_PARENT_DEFERRED",
  "input_contract": {
    "component": "linear E2 ComponentStageProposal with 15x1536 finite component output and complete after-state",
    "lfe": "linear E3 LfeStageProposal with complete ring after-state, explicit DirectLfeStageLineage, and optional Type10LfeRow0Input",
    "parent": "SpatialAudioJoinMetadata and explicit CallbackPhase are supplied as expected typed parent identity from the enclosing coordinator; both proposal lineages must match programme, attempt, transaction, callback, phase, profile, spatial_audio before generation/digest, and shared spatial_control provider lineage",
    "phase": "explicit expected NORMAL or CACHED_DRAIN; never inferred from callback index or row content",
    "provider_lineage": "preserve both E2 F04 parameter-provider lineages and the common F01E/F03 spatial_control provider lineage in the immutable assembly receipt"
  },
  "mapping_document": "docs/implementation/TYPE10-ASSEMBLY-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "not_run": [
    "F04 callback integration and parent transaction fault injection",
    "receiver/QMF",
    "real ADM/audio/render/listening acceptance",
    "W04 product acceptance",
    "full song output"
  ],
  "output_contract": {
    "capability": "consume both input proposals once; packet has no public/raw/default constructor, Clone, Copy, or persistent owner field",
    "commit_authority": "none; the parent callback owns any later root replacement",
    "outcome_discriminant": "Type10AssemblyOutcome::Priming/Ready carries the outcome; Type10AssemblyReceipt contains lineage, origins, and fixed shape only",
    "packet_payload": "private immutable Arc<[FiniteF32]> of exactly 16*1536 samples; expose borrowed read-only samples only",
    "priming_outcome": "Priming plus immutable receipt and both preserved after-states; no packet",
    "ready_outcome": "opaque non-Clone Type10Packet plus immutable receipt and both preserved after-states",
    "receipt_fields": [
      "programme_id",
      "attempt",
      "transaction_id",
      "callback_index",
      "CallbackPhase",
      "selected profile",
      "spatial_audio before generation and digest",
      "spatial_control before generation and digest",
      "spatial_control provider lineage",
      "two F04 parameter-provider lineages",
      "validated LFE host identity",
      "component origin callback g",
      "optional LFE origin callback g-2",
      "fixed 16x1536 shape"
    ],
    "stage_type": "linear Type10AssemblyProposal"
  },
  "overall_p1_3_e_status_remains": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/direct_lfe.rs",
    "crates/jocforge-domain/src/spatial/spatial_control.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs"
  ],
  "planned_test_vector_fixtures": [
    "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json"
  ],
  "programme_root": "spatial_audio",
  "required_e2_lineage_seam": {
    "current_gap": "ComponentStageReceipt does not retain SpatialAudioJoinMetadata.before_generation or before_digest and exposes no read-only lineage accessors",
    "minimal_change": "store the E2 spatial_audio before generation and digest in ComponentStageReceipt and expose crate-internal read-only getters; do not change E2 arithmetic or commit authority"
  },
  "reset_authority": "none; programme-origin reset remains with the enclosing owner coordinator",
  "ring_agreement": {
    "priming": "for g<2, row0 is absent and the ring after-state remains the only LFE result",
    "required": "for g>=2, row0 callback_generation equals g, origin_generation equals g-2, and exactly one matching after-ring slot has that origin and identical row bytes"
  },
  "scope": "Consume one same-callback E2 ComponentStageProposal and E3 LfeStageProposal, stage Type10 row assembly for receiver handoff, and preserve both after-states for the parent. Do not implement receiver/QMF, local commit, parent transaction, JOC, or codec.",
  "specification": {
    "clauses": [
      "F04C.3",
      "F04C.7"
    ],
    "package_manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a",
    "priming": "g=0 and g=1 return Priming and no Type10Packet; g>=2 requires one row0 with origin g-2",
    "receiver_qmf": "not implemented by E4",
    "rows": "Type10[g] is F32[16][1536]; row0 is LFE[g-2] and rows1..15 are component15[g][0..14], copied in row-major order without numeric transformation",
    "spec_bytes": 9103,
    "spec_file": "specs/04-whole-spatial-chain-composition.md",
    "spec_sha256": "92325b12128023001e1f3fd4696dcec68bbd47a46acdb7886781b411a10c5d59"
  },
  "stage_authority": "F04C E4 callback-local assembly only; consumes E2/E3 proposals and preserves their after-states",
  "status": "LOCALLY_VERIFIED_TYPE10_ASSEMBLY__RECEIVER_PARENT_DEFERRED",
  "task_id": "P1.3-E4-F04-TYPE10-ASSEMBLY",
  "test_files": [
    "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
    "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs"
  ],
  "test_inventory": [
    {
      "bytes": 23774,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/type10_assembler/tests.rs",
      "sha256": "5f83ce89089916e9f588c7b80313234d2d5c999fd527c838f5c5da60d7456964"
    },
    {
      "bytes": 51413,
      "path": "crates/jocforge-domain/src/spatial/spatial_audio_state/component_stage/tests.rs",
      "sha256": "b501289f41cc2c08dcbc31e4fb9419b4c79c2383b01af2c7eca864dcfd9ab123"
    }
  ],
  "vectors": {
    "binding_rule": "E4 asserts every output sample bit equals the corresponding consumed proposal row bit; no whole-chain Type10 expected hash is used as an assembly input or oracle",
    "callbacks": [
      "g0",
      "g1",
      "g2",
      "g3",
      "g4",
      "g5"
    ],
    "cases": [
      "g0_g1_priming_without_packet",
      "g2_g5_row_order_origins_and_bit_preservation",
      "component_lfe_programme_attempt_mismatch",
      "callback_phase_mismatch",
      "transaction_mismatch",
      "profile_mismatch",
      "provider_lineage_mismatch",
      "stale_spatial_audio_before_generation",
      "stale_spatial_audio_before_digest",
      "missing_row0_after_priming",
      "row0_origin_or_ring_slot_mismatch",
      "malformed_component_output_shape",
      "dropped_stage_preserves_committed_root"
    ],
    "component_fixture": "E4 test-only ComponentStageProposal rows with unique finite F32 bit markers; E4 makes no E2 DSP/vector claim",
    "expected_values_are_assertions_only": true,
    "lfe_fixture": "crates/jocforge-domain/tests/fixtures/direct_lfe/INPUT-ONLY-VECTORS.json; consume only the g0..g5 callback/phase/origin assertions"
  },
  "verification": {
    "boundary_normal": "PASS",
    "boundary_optimized": "PASS",
    "cargo_check": "PASS",
    "cargo_clippy": "PASS",
    "cargo_fmt": "PASS",
    "full_repository_boundary_mutations_rejected": 685,
    "implementation_map_render": "PASS",
    "local_commit": "NOT_IMPLEMENTED",
    "parent_transaction": "NOT_IMPLEMENTED",
    "planned_test_count": 13,
    "receiver_qmf": "NOT_IMPLEMENTED",
    "targeted_implementation_mutations_rejected": 36,
    "targeted_mapping_mutations_rejected": 20,
    "type10_assembly_test_count": 13,
    "type10_packet": "LOCALLY_VERIFIED_CALLBACK_LOCAL_NONCLONE",
    "workspace_test_ignored": 1,
    "workspace_test_passed": 209
  }
}
```
