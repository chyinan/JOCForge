# F06NR frozen Python reference fingerprints

Status: `MODEL_REFERENCE_ONLY_NOT_INDEPENDENT_GOLDEN`.

This package provides hashes, shapes and structural checks for one frozen Python-model execution over the three already washed synthetic source-frame input recipes. It is useful as a local comparison reference when the implementation runs the same deterministic inputs. It is not a new algorithm specification, an independent oracle, a native DEE result, or a product acceptance test.

The nine F32LE result files (forward history, inverse carry and secondary carrier for source frames 0, 1 and 2) remain in the research package and are deliberately not copied here. This package carries their byte counts and SHA-256 values only. It contains no raw PCM, no sample arrays, no code, no DLL, and no private key. The exact input recipes and input hashes remain in `F06NR-SOURCE-FRAME-INPUT-ONLY-CROSSWALK-V1`.

If implementation output differs from these frozen-model hashes, use the approved per-operation rules and compare the first differing lane/block/value. Do not treat the hash alone as proof that the implementation is wrong or that the frozen model matches a native run. All numeric reference outputs remain graded `FROZEN_PYTHON_REFERENCE_EXECUTION`; independent implementation comparison and product playback are `NOT_RUN`.
