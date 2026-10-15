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
    "implementation_status": "NOT_STARTED",
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
    "materialization_status": "MAPPING_COMPLETE_BLOCKED_ON_P1.4_PROVIDER_CONTRACT",
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
    "implementation_status": "NOT_STARTED",
    "prerequisite": "P1.4-A-F01E-PROVIDER-CONTRACT LOCALLY_VERIFIED",
    "programme_root": "frontend_control",
    "status": "BLOCKED_ON_P1.4_F01E_PROVIDER_CONTRACT",
    "task_id": "P1.3-B-FRONTEND-CONTROL-AUDIT"
  }
}
```
