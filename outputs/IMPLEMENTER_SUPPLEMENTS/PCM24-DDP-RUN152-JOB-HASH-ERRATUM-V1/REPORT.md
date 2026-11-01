# Run152 job-hash correction

This is a Wash-only correction to the previously washed Run152/Run153 storage-address-alias observation. The original Run152 receipt used Run153's job XML hash. The hash-bound erratum corrects only that mapping:

- Run152 correct job XML SHA-256: `31e642109943bc9a93a1704b8a5c6137eb6a50dfaa4e782f0b5dc2c897a8dd9c`.
- Run153 job XML SHA-256: `c1440b3ab909ec59574fb3bede15bc047e9b61428162e8cbb2c9d63d4c459cf6`.

No pointer observation or stack-role conclusion changes. The media hash remains only a Run145 preflight reference; Run152/153 media was not rehashed, so sample identity remains `UNBOUND/NO`. The numeric stack token equality remains address reuse, not proof of a shared object, owner, or generation. Output parsing/playback remain `NOT_RUN`. This correction does not change audio algorithm behavior or implementation readiness.
