# custom_dialnorm owner and default boundary

## Disposition

OVERRIDE_OWNER_AND_DEFAULT_BOUND; DI_ARM_RUNTIME_BRANCH_UNOBSERVED; PCM_CAUSE_UNBOUND.

The addendum binds the optional integer field and its direct store into the per-frame record. It does not establish that the DI false/true PCM difference was caused by this override.

## Bound field path

FUN_180135d40 constructs the dee::proc::Priv root and invokes FUN_180136220 with root +0x20. The sealed registration trace places custom_dialnorm at nested +0xa68, corresponding to root +0xa88. Its integer schema is [-31,0], with 0 as the no-override default.

FUN_180152dd0 is the SafeType<int64,Integer> value getter. FUN_1801535b0 reads root +0xa88 and, when nonzero, converts the integer to float and writes it to source record +0x114; the same routine also changes adjacent trim fields. The runtime value and branch execution were not captured. A prior sealed package maps +0x114 to DDPJocEnc +0x54 and the DDPi DialNorm parameter route; that is reused, not retraced here.

Wash rehashed the current DLL, all eleven source-package files, SHA256SUMS, three parent seals, three parent artifacts, and both controlled job XMLs. Both XMLs are hash-matched, contain dialogue_intelligence, and contain no custom_dialnorm element. This supports that the tested inputs did not request the override; it does not reveal runtime state. The observed PCM hashes differ in the prior receipt, but the override does not explain that difference on the recorded inputs. Overall PCM causality remains UNBOUND.

The source agent reports normal and python -O checker PASS; Wash did not execute it. No DEE rerun, decode, PCM/payload/key/license read, or implementation contact occurred.

## Implementation boundary

The registered default is zero and a supplied nonzero integer reaches record +0x114. Keep downstream trim/audio effect and runtime getter value separate from this static field path.
