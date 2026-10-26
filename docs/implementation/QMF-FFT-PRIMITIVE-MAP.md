# QMF FFT primitive map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Status: `LOCALLY_VERIFIED_FIXED64_ONLY`. This covers only the fixed QMF FFT primitive; QMF callers and whole-chain execution remain separate.

The primitive is a 64-point complex transform over 128 interleaved F32 scalar slots. The map keeps input normalization and caller reorder/sign-bit operations outside the butterfly contract.

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
  "historical_implementation_inventory": [
    {
      "bytes": 3812,
      "path": "crates/jocforge-domain/src/spatial/qmf_fft64.rs",
      "sha256": "d393d09d14346d9c275650ea2918b6e59a18d5113a82d1a9208d9ed044923f18"
    },
    {
      "bytes": 2855,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "91beefcd535b04930a7c3dd2427c081fc114053d44bb3971edb220b924412ba0"
    }
  ],
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
      "bytes": 3261,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "521768b37a8fa6957abc2e55b5383c1b6c363f3356ac46a0a05f77fd6e2bcfe3"
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
    "full_repository_boundary_mutations_rejected": 772,
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
