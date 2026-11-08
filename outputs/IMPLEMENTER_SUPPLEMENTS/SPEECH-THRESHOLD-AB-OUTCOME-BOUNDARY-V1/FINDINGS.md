# Findings

The prior E-373 authorization was used for exactly two N36 arms: speech_threshold 0 and 100. The preflight parent binds the same authorized ADM input, dialogue_intelligence=true, and metering_mode=1770-4.

Both runs started and exited naturally with code 10, emitted zero observer events, and produced no EC-3 output. Original receipt/status files are hash-bound only, not copied or modified; the raw module_base value is omitted.

The experiment is inconclusive. speech_threshold effect, selected owner/generation, runtime scalar result, and exit cause remain UNBOUND/NOT_CAPTURED. Do not treat these runs as PASS, evidence of a specific threshold effect, or proof the option is unsupported. No global absence claim is made.
