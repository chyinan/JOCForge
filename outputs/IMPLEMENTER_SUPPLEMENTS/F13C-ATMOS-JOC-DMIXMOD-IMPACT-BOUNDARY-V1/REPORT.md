# F13C dMixMod on the selected Atmos JOC route

The selected attempt-10 JOCForge job template explicitly contains `loro` and
four `-3` mix levels; no user prompt or active DME preference supplies them.
The current Rust source maps those values to BSI `dmixmod=2` and four level
codes `4`. This is the JOCForge source path, not a DEE DSP behavior claim.

For the hash-bound `dee_ddpjoc_encoder.exe` Ghidra project, `dmixmod` and the
four levels are parsed and validated. In the traced selected config branch,
the first consumer is a generic name/value/error result-tree builder with no
direct downmix field reads; no audio DSP/postmix consumer was found in this
scoped path. The validator's accepted text tokens do not establish a typed
native enum or audible effect. The targeted `.rdata` scan found no literal
Atmos route strings, which does not exclude dynamic routing or another module.

Implementation rule: do not ask the user; keep the selected template values
in the JOCForge source path and do not infer native defaults or audio effects
from labels. This boundary does not block best-effort MVP. DEE encoder and
media were not run/read; native audio impact remains UNBOUND.
