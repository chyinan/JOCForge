# F13 preferred-input decision supersession

Status: `USER_DECISION_SUPERSEDES_EXPLICIT_JOB_FIELD`; `PREFERRED_PROVIDER=UNBOUND`.

The user's latest direction states that the DEE task XML does not request F13.6 `preferred` and that a new task/UI input must not be introduced merely to satisfy the model signature. This supersedes the earlier Wash note that treated it as an explicit per-task user input.

The approved F13.6 model contract still requires a boolean in its internal ID1 configuration. No approved mapping from DEE task XML, F06 `gating_kind`, or BSI `preferred_downmix_mode` to that boolean has been established. Do not add a new user input, default true/false, or substitute a similar field. Keep the value/provider unresolved until the bounded caller/source investigation returns. The clean model field contract remains distinct from proof of the selected native caller.
