# QMF analysis/synthesis core map

由 IMPLEMENTATION-MAP.json 生成，不得手工修改。

Status: `QMF_CORE_IMPL_WITH_DIRECT_VECTORS__WHOLE_RECEIVER_NOT_RUN`. This is a docs-only readiness gate for the selected fixed QMF core; it does not promote the F04 whole root.

## Processing boundary

Analysis accepts one 64-sample lane slot, that lane’s nine-row history, the selected prototype and rotation, and the fixed FFT provider. The caller has already produced `s09_input`; lane routing and receiver preprocessing stay outside this core.

The receiver mode=1 path applies the ten-slot delay and band0 FIR only to supplied surround lanes. It then performs the inverse reorder, unscaled FFT, postmodulation, and synthesis polyphase step. The core returns after-state slices; only `spatial_audio` may commit them later.

## State and clocks

`ModelState.analysis_history` owns five independent histories in `[L,R,C,Ls,Rs]` order. Receiver mode=1 delay, FIR, and synthesis histories are two independent rows for `[Ls,Rs]`. `QmfWindowState.fitting_delay/fitting_fir` remain separate and mode=0 fitting is excluded.

The parent callback uses `FrontendCallbackIndex g`. The receiver uses its separate `ReceiverCallbackCount r`: no receiver/QMF call for `g<2`, then `r=g-2` and `receiver_generation=r+1`. Each receiver callback processes 24 ordered QMF slots; EOF adds no synthetic QMF flush.

## Provider and vector evidence

The qmf functional provider supplies the selected QWIN and mode=1 band0 taps. The mathematical FFT provider supplies the 63 roots and separate 128-word rotation. `receiver_blend32` is an upstream F04R.5 dependency; QMF does not consume it directly. The current fresh review is static only. Direct unit vectors cover FFT, reorder, synthesis history and surround delay/FIR, while fold/rotation and postmodulation lack isolated direct cases; model-first whole-binding hashes remain assertions, not execution evidence for this implementation.

The machine-readable record below binds the exact formulas, providers, owners, clocks, vectors and exclusions.

```json
{
  "aggregate_readiness_unchanged": {
    "F04Q": "DEFERRED_DETAILED_MAPPING",
    "F04QF": "DEFERRED_DETAILED_MAPPING",
    "P1.3-E": "BLOCKED_ON_F04_DSP_RENDER_AND_PARENT",
    "spatial_audio_root": "MAPPING_COMPLETE_BLOCKED"
  },
  "algorithm_contract": {
    "analysis_fft_and_reorder": "Prepare R32(v/64) for all 128 rotated F32 scalars before the first FFT butterfly. Call the fixed 64-complex FFT with no mode parameter. For p=0..31 and FFT output U as U32: out[4p]=U[2p]; out[4p+1]=U[2p+1] XOR 0x80000000; out[4p+2]=U[126-2p]; out[4p+3]=U[127-2p]. XOR is bitwise and must preserve signed-zero/subnormal bits.",
    "analysis_fold": "For k=0..63: e=(H[6,k]*P[128+k])+(H[8,k]*P[k]); then e+=(H[4,k]*P[256+k]); e+=(H[2,k]*P[384+k]); e+=(H[0,k]*P[512+k]); o=(H[5,k]*P[192+k])+(H[7,k]*P[64+k]); then o+=(H[3,k]*P[320+k]); o+=(H[1,k]*P[448+k]); o+=(x[k]*P[576+k]); store fold[126-2*k]=o and fold[127-2*k]=e. Each term follows the displayed left-to-right R32 order.",
    "analysis_input": "One lane supplies x[0..63], H[0..8][0..63] from that lane's before analysis history, qwin_synthesis_640[0..639], rotation[0..63] as 64 complex pairs, and the validated fixed-64 FFT provider.",
    "analysis_next_history": "After producing all 128 fold scalars, next H=[x,H[0],H[1],...,H[7]] by raw F32 bits; advance once per 64-sample slot, not once per callback.",
    "analysis_rotation": "For folded complex (r,i) and rotation (a,b): rotated.real=(i*a)-(r*b); rotated.imag=(i*b)+(r*a), with each operation independently R32. Rotation source is math provider rotation_u32, not FFT stage roots.",
    "fft_primitive_binding": {
      "fixed_contract": {
        "arithmetic": "each operation separately R32; no FMA or normalization",
        "butterfly_destinations": "write out_a to work[left] and out_b to work[right]",
        "butterfly_indexing": "For stage L, half=L/2; left=base+j and right=base+half+j; read a=work[left] and b=work[right] before either write; roots[j][0] is real c and roots[j][1] is imaginary d",
        "butterfly_order": [
          "tr=(c*b_re)-(d*b_im)",
          "ti=(c*b_im)+(d*b_re)",
          "out_a=(a_re+tr,a_im+ti)",
          "out_b=(a_re-tr,a_im-ti)"
        ],
        "complex_points": 64,
        "exponent": "negative",
        "input_bit_reversal": "work[k]=input[rev6(k)]; preserve each real/imag pair",
        "interleaved_f32_slots": 128,
        "output": "64 complex bins in natural order",
        "stage_iteration": "L ascending; base=0,L,...,64-L; j=0..L/2-1; roots[j] from the current stage",
        "stage_lengths": [
          2,
          4,
          8,
          16,
          32,
          64
        ]
      },
      "implementation_file": "crates/jocforge-domain/src/spatial/qmf_fft64.rs",
      "mapping_document": "docs/implementation/QMF-FFT-PRIMITIVE-MAP.md",
      "status": "LOCALLY_VERIFIED_FIXED64_ONLY",
      "typed_api": "QmfFftInput::new(&[f32]) -> Result<QmfFftInput,QmfFftError>; execute_qmf_fft64(&QmfFftInput,&QmfMathematicalFftProvider) -> Result<QmfFftOutput,QmfFftError>"
    },
    "numeric_rule": "Every F32 multiply/add/subtract is one R32 round-to-nearest-ties-to-even point; do not fuse, regroup, or change loop order. Preserve signed zero and gradual underflow. Nonfinite intermediate or output is fatal per N00.2.",
    "prototype_binding": "synthesis P[j]=qwin_synthesis_640[j]; analysis P[j]=qwin_synthesis_640[639-j]. The analysis reverse is an index reversal only, not a second provider array.",
    "receiver_mode1_delay_and_fir": "For a supplied surround lane's 24-slot analysis X[24][128], join old D[10][128]||X and use the first 24 slots as delay output. In mode=1 each delayed complex band (r,i) becomes (i XOR 0x80000000,r); next D is the last 10 slots of the unrotated X. Band0 FIR reads current X band0, not the delay output: A=old 20 complex samples||X band0. For s=0..23, initialize ar=ai=+0 and for t=0..20 ascending let (x,y)=A[s+t], ar=(ar+(R[t]*x))-(I[t]*y), ai=(ai+(R[t]*y))+(I[t]*x). Replace band0 with (2*ar,2*ai); other 63 bands retain the delay output. Next FIR history is the last 20 complex samples of A.",
    "route": "QMF_FFT_RADIX2_MATH_V1",
    "synthesis_fft_and_postmodulation": "Call the same fixed 64-complex negative-exponent FFT on V without 1/64 input scaling. For each output (r,i) and rotation (a,b): real=((r*b)+(i*a))*2; imag=((i*b)-(r*a))*2. Every multiplication, addition, subtraction, and factor-of-two multiplication has a separate R32 point.",
    "synthesis_polyphase": "For k=0..63 let poly_group=floor(k/4), lane=k mod 4, h=36*poly_group+lane; all right-hand H references use the old 576-word snapshot. v=(P[k]*r[k])+H[h]; output[k]=v+v. For j=0..7, c=i[k] when (j+1) is odd, otherwise r[k]; nextH[h+4*j]=(P[64*(j+1)+k]*c)+H[h+4*(j+1)]. Then nextH[h+32]=P[576+k]*i[k]. This produces 64 samples per slot and advances one synthesis history per processed slot.",
    "synthesis_preformat": "For each processed band slot, with reordered input U as U32, p=0..31: V[2p]=U[4p]; V[2p+1]=U[4p+1] XOR 0x80000000; V[126-2p]=U[4p+2]; V[127-2p]=U[4p+3]. This is the inverse analysis bit permutation."
  },
  "callback_clock": {
    "callback_advance": "ReceiverCallbackCount advances once per successful receiver callback; each history after-slice is handed to the existing spatial_audio owner. QMF_CORE has no callback commit authority.",
    "eof": "Follow actual parent callback delivery and cached drains; do not append a QMF flush callback.",
    "frontend": "FrontendCallbackIndex g is the parent callback identity; phase comes from the parent and is not inferred from content.",
    "receiver": "ReceiverCallbackCount r is distinct from g. For g<2 Type10 is priming and receiver/QMF does not run; for g>=2, r=g-2 and receiver_generation=r+1. QMF_CORE does not derive either clock from hashes or sample values.",
    "slot_advance": "Each receiver call consumes 24 ordered 64-sample slots. All five analysis histories advance per slot; the two receiver mode=1 surround delay/FIR/synthesis families advance only for lanes [4,5].",
    "slot_count": 24
  },
  "consumer_scope": {
    "excluded": [
      "receiver Type10 routing",
      "F04R stages before s09_input",
      "descriptor",
      "F05JC fitting",
      "J05 owner receipt",
      "F04C parent commit",
      "real ADM/audio/listening acceptance"
    ],
    "fitting_mode0": "EXCLUDED. F04RB.2 has five independent fitting delay/FIR owners; mode=0 updates fitting delay and 20-complex history, skips tap multiplication, does not overwrite band0, and has no synthesis FFT. It must not alias receiver mode=1 state or be implemented in QMF_CORE.",
    "mode0_vector_boundary": "F04RB-MODE0-FITTING-V1 contains direct mode0 fitting KATs, owned by MODE0_FITTING_AUDIT and not QMF_CORE. U-QMF-SURROUND-RESET remains mode=1 only; whole-binding hashes remain model-first assertions.",
    "qmf_analysis_history_row_order": [
      "L",
      "R",
      "C",
      "Ls",
      "Rs"
    ],
    "receiver_analysis_lanes": [
      0,
      1,
      2,
      4,
      5
    ],
    "receiver_blend32": "Upstream dependency only. parameters/window-q15.json receiver_blend32[32] supplies F04R.5 before s09_input; QMF_CORE does not consume it directly. component_kernel64[64] belongs F04A.8 and is excluded.",
    "receiver_input_boundary": "Receiver supplies same-generation s09_input after F04R stage 3; QMF_CORE starts at the 64-sample lane/slot and does not produce s09_input or choose lane routing.",
    "receiver_mode1_synthesis_lanes": [
      4,
      5
    ],
    "receiver_surround_history_row_order": [
      "Ls",
      "Rs"
    ],
    "six_lane_order": [
      "L",
      "R",
      "C",
      "LFE",
      "Ls",
      "Rs"
    ]
  },
  "error_contract": {
    "invalid_mode1_band0_taps": "RECEIVER_PARAMETER_INVALID",
    "invalid_prototype_length_or_finite_value": "QMF_PROTOTYPE_INVALID_INPUT",
    "invalid_shape_or_nonfinite_input": "reject before processing per F04Q.1/I03; no normal output and no accepted after-state",
    "missing_mode1_band0_taps": "RECEIVER_PARAMETER_UNAVAILABLE",
    "missing_prototype": "QMF_PROTOTYPE_UNAVAILABLE; no unit/identity table fallback",
    "native_error_claim": false,
    "nonfinite_intermediate_or_output": "fatal F04_NUMERIC_FAILURE per N00.2; no normal output or accepted after-state",
    "outer_policy": "the enclosing F04/I03 callback stops and applies programme retirement; this mapping does not implement or prove parent rollback",
    "prototype_identity_mismatch": "QMF_PROTOTYPE_IDENTITY_MISMATCH",
    "state_effect": "all before slices remain caller-owned and unchanged; QMF_CORE has no local commit, abort, or fatal authority",
    "typed_fft_or_provider_failure": "return the existing F04ParameterProviderError before QMF work; no native error-code mapping claim"
  },
  "fixture_files": [
    {
      "bytes": 48429,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/analysis/F04Q2-ANALYSIS-DIRECT-INPUTS.json",
      "sha256": "5a05987ec1d2bd3707234d2d7842f7ad8d6e8e18f9d549266ad2447c1aa0b229"
    },
    {
      "bytes": 144743,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/analysis/F04Q2-ANALYSIS-DIRECT-ACTUAL.json",
      "sha256": "24bf73c9d5f27f5bd4938155920edc9933b8d7d967e4d2ef84d4154f8515840f"
    },
    {
      "bytes": 1741,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/analysis/F04Q2-INDEPENDENT-CHECK.json",
      "sha256": "385f6589dff66c323a4e8e83804343bcb8490a0d1691c860fb51a88fa97f0529"
    },
    {
      "bytes": 1208,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/analysis/SPEC.md",
      "sha256": "c67f906eb3cae1f7243696f30f9cd01641883d64c6a5d66afb281a84673ae305"
    },
    {
      "bytes": 1130988,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/synthesis/F04Q5-SYNTHESIS-DIRECT-LITERALS.json",
      "sha256": "36f4ba70a9840701f56ae2c90e53b325c390de79b8a7e4b616990f558d4f5843"
    },
    {
      "bytes": 1087,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/synthesis/ACTUAL-RESULT.json",
      "sha256": "3b2e494c3b2632afbc53f94e14c93cb6e8ca2f144d4b9c3d760c263cbde2fc20"
    },
    {
      "bytes": 936,
      "file": "crates/jocforge-domain/tests/fixtures/qmf_direct/synthesis/SPEC.md",
      "sha256": "3c39ac083d0dd00e0d243580e0bf95d08fa07178b699b3b056acdb7caea35d27"
    }
  ],
  "functional_id_scope": "The selected fixed QMF analysis/synthesis core and its receiver mode=1 state slices are locally implemented and checked against direct literal vectors. Whole F04Q/F04QF functional IDs, receiver routing, mode=0 fitting, descriptor/JOC, J05, F04C parent commit, real ADM and product acceptance remain separate or blocked.",
  "functional_ids": [
    "F04Q",
    "F04QF",
    "F04QP",
    "F04RB"
  ],
  "implementation_files": [
    "crates/jocforge-domain/src/spatial/qmf_analysis_synthesis.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "implementation_inventory": [
    {
      "bytes": 21790,
      "path": "crates/jocforge-domain/src/spatial/qmf_analysis_synthesis.rs",
      "sha256": "9184b8ca22b5ed89082a92e3027b74c13d7e5424c4504598c3e71c239f2beeaa"
    },
    {
      "bytes": 2206,
      "path": "crates/jocforge-domain/src/spatial/mod.rs",
      "sha256": "4559a9054f74e51a5436681b6362b292248476262ca1dd94b6496d2f0f79a9b6"
    }
  ],
  "implementation_prohibitions": [
    "no generic QMF or configurable channel-count plan",
    "no receiver Type10 routing or s09_input production",
    "no F04R transform/target/descriptor pipeline",
    "no mode=0 fitting state or fitting algorithm",
    "no J05, descriptor, JOC, render, or parent CallbackTransaction",
    "no local state commit/abort/fatal authority",
    "no public or third-party provider redistribution",
    "no whole receiver/J05/DSP pipeline execution or product/audio acceptance claim"
  ],
  "implementation_status": "QMF_CORE_IMPL_WITH_DIRECT_VECTORS__WHOLE_RECEIVER_NOT_RUN",
  "mapping_document": "docs/implementation/QMF-ANALYSIS-SYNTHESIS-CORE-MAP.md",
  "mapping_status": "MAPPING_COMPLETE",
  "module_export_file": "crates/jocforge-domain/src/spatial/mod.rs",
  "package_identity": {
    "manifest_bytes": 76302,
    "manifest_file": "MANIFEST.json",
    "manifest_sha256": "c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a"
  },
  "planned_implementation_files": [
    "crates/jocforge-domain/src/spatial/qmf_analysis_synthesis.rs",
    "crates/jocforge-domain/src/spatial/mod.rs"
  ],
  "planned_test_files": [
    "crates/jocforge-domain/src/spatial/qmf_analysis_synthesis/tests.rs"
  ],
  "programme_root": "spatial_audio",
  "provider_bindings": {
    "qmf_mathematical_fft": {
      "bytes": 6983,
      "file": "parameters/qmf-mathematical-fft-v1.json",
      "roots": "stages[].length and stages[].roots_u32[][0..1]; six stage lengths [2,4,8,16,32,64], root counts [1,2,4,8,16,32], 63 pairs total",
      "rotation": "rotation_u32[128] = 64 interleaved F32 words; distinct from root pairs",
      "route": "QMF_FFT_RADIX2_MATH_V1",
      "sha256": "e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431",
      "typed_provider": "QmfMathematicalFftProvider"
    },
    "qmf_receiver_functional": {
      "bytes": 20325,
      "excluded_fields": [
        "lfe_post_filter_4x5",
        "descriptor_threshold_127",
        "12 F04R/F04D scalars"
      ],
      "fields": [
        {
          "consumer": "synthesis QMF prototype",
          "count": 640,
          "f32le_sha256": "870d3c80e4fe7ee368b5da1f7db487dfa84be44c4cf8c82813b78fb06a8e94a2",
          "name": "qwin_synthesis_640"
        },
        {
          "consumer": "analysis QMF prototype; derived reverse, not a second delivered array",
          "count": 640,
          "f32le_sha256": "512b2a9f8be3b98afb7fd1bbd7acb777fac06dd5b98eec58ee5e6672c9a58a49",
          "name": "qwin_analysis_640",
          "rule": "qwin_synthesis_640[639-j]"
        },
        {
          "consumer": "receiver mode=1 band0 FIR",
          "count": 21,
          "f32le_sha256": "b352accaefeb0ff0bac1981dcbe9112ff9bd9437e14f1662076b41cf9fd70c70",
          "name": "band0_real_mode1_21"
        },
        {
          "consumer": "receiver mode=1 band0 FIR",
          "count": 21,
          "f32le_sha256": "946593788adebf7dd8beafb057c77e167e5b066607306d2a4e31d99244b8b3be",
          "name": "band0_imag_mode1_21"
        }
      ],
      "file": "parameters/qmf-receiver-functional-v1.json",
      "sha256": "30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c",
      "typed_provider": "QmfReceiverFunctionalProvider"
    },
    "validation": "Use the existing identity-validated manifest and typed validators before consumer calls; missing or mismatched selected providers fail closed. Do not parse provider JSON in QMF_CORE or regenerate the supplied FFT roots/rotation.",
    "window_q15": {
      "bytes": 2704,
      "direct_qmf_core_consumption": false,
      "excluded_field": "component_kernel64[64] belongs to F04A.8",
      "file": "parameters/window-q15.json",
      "necessary_upstream_field": {
        "count": 32,
        "f32le_sha256": "3968b2666444f0395b008e8d8eb9d84eb2663b4d11199a486b8cbadcf5c142b2",
        "generation": "RNE_integer(32768*sin((j+1/2)*pi/(2*32))^2)",
        "name": "receiver_blend32",
        "representation": "Q15 u16",
        "to_F32": "R32(q15_value/32768), exact",
        "use": "F04R.5 transform delay/double-end blend before the QMF input boundary"
      },
      "sha256": "a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130"
    }
  },
  "readiness_basis": "The fixed analysis/synthesis formulas, operation order, typed parameter identities, state shapes/cold values, caller boundary, callback/slot clocks, and fatal outcomes are explicit in bound specs. The isolated implementation now matches washed direct stage literals for analysis, synthesis, and one mode=1 24-slot callback. This supports only QMF_CORE direct-vector implementation status; whole F04 receiver integration and all product acceptance remain NOT_RUN.",
  "review_evidence": {
    "current_fresh_review": {
      "checks": "44/44",
      "dsp_execution": "NOT_RUN",
      "kind": "STATIC_ONLY",
      "manifest_sha256": "75e7313563c37f79a9afb895704b5eb77e6dda55d41117f820e62c445bbe4999",
      "review_sha256": "926c8a3c9cd21e68665ac287ad98403481b55adf3cdeef6db7660c0f4fb37a60",
      "source": "outputs/IMPLEMENTER_SUPPLEMENTS/F04-CURRENT-FRESH-REVIEW-R1",
      "status_sha256": "4aa0f1d9f0fbc718772193159069077618e0c0138c8002798c893d29f0296fb8"
    },
    "f04_consumer_r1_attestation": {
      "implementation_ready": "NO",
      "numeric_recomputation": "PASS for six synthetic frontend callbacks and four receiver/fitting calls",
      "qualification": "whole-spatial consumer attestation; not product execution",
      "source": "test-vectors/F04-WHOLE-SPATIAL-CONSUMER-LOCAL-REVIEW-STATUS.json"
    },
    "qmf_direct_vectors": {
      "analysis_manifest_sha256": "79830470e3101a10d96852f44c855252c7b78533e0c2b32a73c86d93f74542c6",
      "qualification": "washed stage literals assert local QMF core computations after fresh execution; whole receiver, J05/parent, real ADM, and listening remain NOT_RUN",
      "synthesis_manifest_sha256": "bff23be87f4b7ceb27c145ab6106dd1b786aa2e6dbc35d1e4892da73743e9f4a"
    },
    "qmf_receiver_r2_attestation": {
      "f04rb_local_ready": "YES",
      "implementation_ready": "NO",
      "numeric_recomputation": "PASS for the scoped synthetic five-callback model",
      "qualification": "model-first package attestation; not current isolated implementation DSP execution",
      "selected_route_certified_by_fresh_checker": false,
      "source": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-LOCAL-REVIEW-STATUS.json"
    }
  },
  "scope": "Implement and locally verify F04Q.2-.6 using the existing QMF_FFT_RADIX2_MATH_V1 transform and validated providers. Accept one lane's 64-sample slot and its caller-owned before histories; return the slot output and proposed after histories. The receiver mode=1 callback accepts one supplied surround lane's 24 ordered analysis slots and returns PCM plus delay/FIR/synthesis after slices. Do not choose lanes, create s09_input, run F04R, compute descriptor, or commit a spatial_audio root.",
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
      "bytes": 5584,
      "clauses": [
        "F04Q.1",
        "F04Q.2",
        "F04Q.3",
        "F04Q.4",
        "F04Q.5",
        "F04Q.6",
        "F04Q.7"
      ],
      "file": "specs/04-qmf-analysis-and-synthesis.md",
      "sha256": "13a2907c8e60bd81f948f33b9ad07f2d7dc0552b0b12f53224d27b80af610edf"
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
      "bytes": 4428,
      "clauses": [
        "F04QP.1",
        "F04QP.3",
        "F04QP.4"
      ],
      "file": "specs/04-qmf-prototype-source-and-loading.md",
      "sha256": "8255e0d95eb91db0c5ecf6b120bab4b7f60796d121bf9249bb51db72bc91811f"
    },
    {
      "bytes": 8332,
      "clauses": [
        "F04R.1",
        "F04R.2",
        "F04R.5",
        "F04R.7"
      ],
      "file": "specs/04-receiver-state-and-routing.md",
      "sha256": "74b9b01537eb76a37582535b84426138ee8c8ecdca3486f9f43d0504c7c26306"
    },
    {
      "bytes": 13595,
      "clauses": [
        "F04RB.1",
        "F04RB.2",
        "F04RB.5",
        "F04RB.6",
        "F04RB.7",
        "F04RB.8",
        "F04RB.9"
      ],
      "file": "specs/04-qmf-receiver-whole-binding.md",
      "sha256": "104d604cf876cc522a85b21f352bd0a09fc20c3171ddcd71da981c8076ea3a75"
    }
  ],
  "state_ownership": {
    "analysis_history": {
      "canonical_field": "ModelState.analysis_history",
      "cold": "all positive zero",
      "lane_order": [
        "L",
        "R",
        "C",
        "Ls",
        "Rs"
      ],
      "shape": "T5<T9<B256>> = 5 lanes x 9 rows x 64 F32",
      "transition": "one [x,H0,...,H7] update per lane per slot"
    },
    "canonical_schema": "test-vectors/F04-WHOLE-SPATIAL-STATE-SCHEMA.json and F04RB.7 encode these fields in the owning ModelState; QMF_CORE introduces no independent serialization root.",
    "commit_authority": "enclosing spatial_audio root/callback transaction only; QMF_CORE returns proposed after slices and cannot commit, abort, or retire a programme.",
    "eof_drain": "process each actual receiver callback and its 24 slots, including frontend cached drains when Type10 is available; no extra QMF zero flush or synthetic callback.",
    "fatal_behavior": "invalid dimensions/nonfinite inputs map through the existing F04_INPUT_ERROR boundary; nonfinite intermediate/output maps to F04_NUMERIC_FAILURE. Produce no normal output or accepted after-state; the enclosing I03/parent owner applies retirement. No native error-code equivalence claim.",
    "fitting_mode0": {
      "canonical_fields": [
        "QmfWindowState.fitting_delay",
        "QmfWindowState.fitting_fir"
      ],
      "owner": "spatial_audio fitting_qmf_state; distinct from ModelState receiver histories",
      "scope": "excluded from QMF_CORE implementation"
    },
    "receiver_mode1": {
      "canonical_fields": [
        "ModelState.qmf_delay_history",
        "ModelState.qmf_fir_history",
        "ModelState.synthesis_history"
      ],
      "cold": "all positive zero",
      "lane_order": [
        "Ls",
        "Rs"
      ],
      "shapes": [
        "T2<T10<B512>> = 2 surround lanes x 10 slots x 128 F32",
        "T2<B160> = 2 surround lanes x 40 F32 = 20 complex",
        "T2<B2304> = 2 surround lanes x 576 F32"
      ]
    },
    "reset_authority": "programme-origin full reset reconstructs all QMF arrays from positive-zero cold state; no local resume after RETIRED.",
    "single_owner": "spatial_audio; QMF_CORE adds no committed-state owner, root, receipt, or local commit authority."
  },
  "status": "QMF_CORE_IMPL_WITH_DIRECT_VECTORS__WHOLE_RECEIVER_NOT_RUN",
  "task_id": "QMF-ANALYSIS-SYNTHESIS-CORE-MAPPING",
  "test_files": [
    "crates/jocforge-domain/src/spatial/qmf_analysis_synthesis/tests.rs"
  ],
  "test_inventory": [
    {
      "bytes": 14759,
      "path": "crates/jocforge-domain/src/spatial/qmf_analysis_synthesis/tests.rs",
      "sha256": "64243b2abd6668d321ed0c093bf1a3075ff71f487b282d123fc081a2cba2f037"
    }
  ],
  "vector_bindings": [
    {
      "bytes": 22205,
      "file": "test-vectors/unit/ADAPTER-CONTRACT.md",
      "scope": "U00.2 adapter protocol and per-entry outputs; runner sends input/attachments, never expected values",
      "sha256": "9bce2e35ce47147c76c13e58ee976d44c5109938f0b19e1f3a81f1d8a61fa1ae"
    },
    {
      "bytes": 1503142,
      "descriptor_cases_excluded": [
        "U-QMF-DESCRIPTOR-CONTINUOUS",
        "U-QMF-DESCRIPTOR-EQUAL",
        "U-QMF-DESCRIPTOR-LAST"
      ],
      "direct_qmf_core_cases": [
        "U-QMF-FFT-ANALYSIS",
        "U-QMF-FFT-SYNTHESIS",
        "U-QMF-REORDER",
        "U-QMF-SYNTHESIS",
        "U-QMF-SURROUND-RESET"
      ],
      "file": "test-vectors/unit/vectors.json",
      "prototype_install_cases": [
        "U-QMF-PROTOTYPE-OK",
        "U-QMF-PROTOTYPE-LENGTH",
        "U-QMF-PROTOTYPE-FINITE",
        "U-QMF-PROTOTYPE-IDENTITY"
      ],
      "qualification": "literal U32 arrays or specified input recipes; expected fields are assertions, never state seeds",
      "sha256": "a14dd6c3700fae1765399c72c816dd3146d57ab7dd026b375efb6aed7408cfa8",
      "total_cases": 202
    },
    {
      "bytes": 1158,
      "file": "test-vectors/QMF-MATHEMATICAL-FFT-VERIFICATION.json",
      "scope": "160 synthetic mathematical cases per FFT mode, including 128 basis cases and 20480 compared values per mode; summary only, not raw per-case arrays; production audio NOT_RUN",
      "sha256": "02d93b89be10293f37ae283b80efcca5dfd434d4fd31b0494855e420db004f2f"
    },
    {
      "bytes": 74454,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-VERIFICATION.json",
      "scope": "five continuous cold input-only callback recipes; expected stage/state/window hashes are assertions, not seeds; 14400 analysis-history joins",
      "sha256": "1d809ed24c805a182e9f2ba668562f1f3c8028e4dfc0ce7cd039e1255e4cda07"
    },
    {
      "bytes": 1468,
      "file": "test-vectors/QMF-RECEIVER-WHOLE-BINDING-RESULT.json",
      "scope": "PASS_SCOPED_MODEL_FIRST, 5 callbacks; not isolated implementation or product acceptance",
      "sha256": "cb42de15ce4fa787aa03e511b9f2c6da39a72d3927f00eb020295af8a02862c7"
    },
    {
      "bytes": 260481,
      "file": "test-vectors/F04-WHOLE-SPATIAL-CHAIN-VERIFICATION.json",
      "scope": "six frontend callbacks, two normal and four cached drains; input-only synthetic recipes; target/carrier QMF windows have whole-chain hashes, not QMF-step literal output",
      "sha256": "ffb1dea3b1e69383312da4059b9d8ddabb4366ed8fc22e7537f750ba041f2d63"
    },
    {
      "actual_file": "crates/jocforge-domain/tests/fixtures/qmf_direct/analysis/F04Q2-ANALYSIS-DIRECT-ACTUAL.json",
      "actual_sha256": "24bf73c9d5f27f5bd4938155920edc9933b8d7d967e4d2ef84d4154f8515840f",
      "cases": [
        "cold_sparse_impulse",
        "signed_zero_reorder_xor",
        "analysis_scale_finite_boundary",
        "warm_history_shift"
      ],
      "input_file": "crates/jocforge-domain/tests/fixtures/qmf_direct/analysis/F04Q2-ANALYSIS-DIRECT-INPUTS.json",
      "input_sha256": "5a05987ec1d2bd3707234d2d7842f7ad8d6e8e18f9d549266ad2447c1aa0b229",
      "kind": "WASHED_DIRECT_ANALYSIS_LITERALS",
      "manifest_sha256": "79830470e3101a10d96852f44c855252c7b78533e0c2b32a73c86d93f74542c6",
      "qualification": "actual-first direct stage assertions for fold, rotation, analysis scale/bitrev, FFT output, reorder, and history; not whole receiver acceptance",
      "source_supplement": "QMF-ANALYSIS-DIRECT-VECTORS-V1"
    },
    {
      "cases": [
        "SYNTH-COLD-SPARSE-SIGNEDZERO",
        "SYNTH-WARM-HISTORY-SHIFT",
        "MODE1-COLD-24-BAND0-FIR"
      ],
      "kind": "WASHED_DIRECT_SYNTHESIS_LITERALS",
      "literal_file": "crates/jocforge-domain/tests/fixtures/qmf_direct/synthesis/F04Q5-SYNTHESIS-DIRECT-LITERALS.json",
      "literal_sha256": "36f4ba70a9840701f56ae2c90e53b325c390de79b8a7e4b616990f558d4f5843",
      "manifest_sha256": "bff23be87f4b7ceb27c145ab6106dd1b786aa2e6dbc35d1e4892da73743e9f4a",
      "qualification": "actual-first direct stage assertions for mode=1 delay/FIR, preformat, inverse FFT, postmodulation, polyphase, and next states; whole receiver/J05 not run",
      "source_supplement": "QMF-SYNTHESIS-DIRECT-VECTORS-V1"
    }
  ],
  "vector_gaps": [
    "No whole F04 receiver integration vector is executed by QMF_CORE. The five-callback whole-binding and six-callback chain hashes remain assertions outside this isolated core test; receiver routing, J05/parent, real ADM, and listening remain NOT_RUN.",
    "The fresh F04-CURRENT-FRESH-REVIEW-R1 remains static-only (44/44); it is not an implementation execution review."
  ],
  "verification": {
    "current_fresh_review": "STATIC_ONLY_44_OF_44",
    "full_repository_boundary_mutations_rejected": 772,
    "listening_spatial_acceptance": "NOT_RUN",
    "model_first_receiver_review": "SUPPLIED_ATTESTATION_ONLY_IMPLEMENTATION_READY_NO",
    "package_spec_provider_vector_identities": "PASS",
    "qmf_core_direct_test_count": 7,
    "qmf_core_dsp_execution": "DIRECT_STAGE_VECTORS_PASS",
    "qmf_core_whole_receiver_execution": "NOT_RUN",
    "real_adm_whole_song": "NOT_RUN",
    "targeted_boundary_mutations_rejected": 44
  }
}
```
